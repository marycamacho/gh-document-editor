# Docs Editor — Product & Technical Spec

A minimal Tauri desktop app that lets non-technical team members edit and create markdown docs in the Cirdia GitHub repo. Every edit lands on its own auto-created branch and arrives as a pull request. The person never sees git.

---

## 1. Goal & Non-Goals

**Goal.** One window. Browse the repo's folder tree, click a doc, click Edit, type, click Save, click Close. The app handles branch creation, commits, push, and PR behind the scenes.

**Non-goals (v1).**
- No local clone or local git. All operations go through the GitHub REST API.
- No merge conflict resolution UI. If the file changed upstream mid-edit, warn and offer "save anyway to my branch" (safe — it's their branch) or discard.
- No image upload, no file rename/delete.
- No arbitrary multi-repo support. The config lists the available libraries (`REPO_OWNER`/`REPO_NAME` plus the optional `REPOS` list), and the footer's library name is a click-to-switch menu across them — one active library at a time, one sign-in covering all of them. Adding a library is a config change, not a UI flow. The two known deployments are **`cirdia-wellness/cirdia-documentation`** and **`marycamacho/writing`**.
- No review UI. Review happens in GitHub as normal.

---

## 2. User Flow

```
Launch app
  └─ Signed in? ──no──► First-run screen: name + "Sign in with GitHub"
  │                     (device code → approve in browser → signed in for good)
  └─ yes
      ▼
Tree view (left sidebar, like an IDE)
  • Shows folders + .md files from default branch (main)
  • Click a file ► read-only preview (rendered markdown)
  • [+ New document] button (sidebar top)
      ▼
[+ New document]
  • Dialog: Title (required) + Folder (dropdown of existing folders)
  • Filename auto-generated from title: <title-slug>.md (shown live, editable)
  • Duplicate check against the tree; block with "a doc with that name exists"
  • Creates branch docs/new-<file-slug>-<YYYYMMDD-HHmm>, opens editor
    pre-filled with "# <Title>" — from here identical to the edit flow
      ▼
[Edit] button
  • App creates branch: docs/<file-slug>-<YYYYMMDD-HHmm>
  • Editor pane opens (markdown source, with HackMD-style Write / Split /
    Preview view modes — Split shows source and rendered preview side by side)
      ▼
[Save] button (can be pressed many times)
  • Commits current buffer to the branch
  • Auto message: "Update <filename> — <display name>, <YYYY-MM-DD HH:mm>"
  • Toast: "Saved ✓"
      ▼
[Close & Submit] button
  • Opens PR: branch → main
  • PR title: "Docs: update <filename> (<display name>)"
  • PR body: file path, edit session start/end times, commit count
  • Confirmation screen: "Submitted for review" + link to the PR
      ▼
[Close without saving] (only if zero commits made)
  • Deletes the branch, returns to tree view

Edge: person closes the window mid-edit with unsaved buffer changes
  ► native confirm dialog: Save / Discard / Cancel
Edge: person closes the window after commits but before submitting
  ► on next launch, app detects their open docs/* branches with no PR
    and offers: "You have an unsubmitted edit to <file> — Resume / Submit / Discard"
```

---

## 3. Branch & Commit Conventions

| Thing | Format | Example |
|---|---|---|
| Branch name (edit) | `docs/<file-slug>-<YYYYMMDD-HHmm>` | `docs/onboarding-guide-20260813-1042` |
| Branch name (new) | `docs/new-<file-slug>-<YYYYMMDD-HHmm>` | `docs/new-offboarding-checklist-20260813-1115` |
| Commit message (new) | `Create <filename> — <name>, <date time>` | `Create offboarding-checklist.md — Ana, 2026-08-13 11:15` |
| PR title (new) | `Docs: new <filename> (<name>)` | `Docs: new offboarding-checklist.md (Ana)` |
| Commit message | `Update <filename> — <name>, <date time>` | `Update onboarding-guide.md — Ana, 2026-08-13 10:47` |
| PR title | `Docs: update <filename> (<name>)` | `Docs: update onboarding-guide.md (Ana)` |
| Merge strategy | Whatever the repo default is; multi-commit PRs are fine | — |

`<file-slug>` = filename lowercased, extension dropped, non-alphanumerics → `-`.
`<name>` comes from `DISPLAY_NAME` in `.env` (fall back to the signed-in GitHub login).
Timestamps in the person's local timezone.

---

## 4. Architecture

```
┌───────────────────────────── Tauri app ─────────────────────────────┐
│                                                                      │
│  Svelte front end (webview)           Rust shell                     │
│  ┌──────────────┐  ┌──────────────┐   • window, menus                │
│  │ Tree sidebar │  │ Editor pane  │   • .env / config resolution     │
│  │ (folders,    │  │ CodeMirror 6 │   • GitHub App device-flow       │
│  │  .md files)  │  │ + preview    │     sign-in; token file (0600)   │
│  └──────┬───────┘  └──────┬───────┘   • GitHub REST client (reqwest) │
│         │                 │           • no git binary needed         │
│         └── invoke (typed commands) ──► Rust ── HTTPS ──► api.github.com
└──────────────────────────────────────────────────────────────────────┘
```

- **Front end:** Svelte 5 + CodeMirror 6 (`@codemirror/lang-markdown`), with Write / Split / Preview view modes (`marked` + DOMPurify for the rendered side). Milkdown is an alternative if a WYSIWYG feel is wanted later; v1 ships source-with-preview.
- **GitHub client:** in the **Rust layer** (`reqwest`), matching the pattern of every app in the ecosystem — all network traffic from Rust. The webview invokes typed Tauri commands; **the token never enters the webview at all** — sign-in happens Rust-side via the device flow, and the webview only ever sees the short user code. Plain-language error mapping stays in the front end, keyed off the structured `{kind, status, message}` errors the Rust side returns.
- **Auth:** GitHub App ("Cirdia Docs Editor", Client ID baked into the build) with **device flow** — the person approves once in the browser with a short code; token expiration is disabled on the App registration, so the sign-in is permanent until revoked. The token is stored in an owner-only (0600) `auth.json` in the app data dir — no OS keychain, so no permission prompts, on any platform, dev or shipped. One sign-in covers every installed library. `GITHUB_TOKEN` in `.env` remains a dev/power-user override.
- **Rust side:** window management, config resolution, device-flow auth + token file, and the GitHub client. No git2, no shell-outs.
- **State:** in-memory + webview localStorage for display name, draft buffers, and session records; the token lives only in the app-data token file (or the `.env` override).

### API call map

| UI action | GitHub REST call |
|---|---|
| Load tree | `GET /repos/{o}/{r}/git/trees/{main-sha}?recursive=1` (filter `.md`) |
| Open file | `GET /repos/{o}/{r}/contents/{path}?ref=main` (keep the blob `sha`) |
| Edit → create branch | `GET /repos/{o}/{r}/git/ref/heads/main` then `POST /repos/{o}/{r}/git/refs` |
| Save | `PUT /repos/{o}/{r}/contents/{path}` with `branch`, `sha`, base64 content, message |
| New file (first save) | Same `PUT /contents` call, **no `sha`** — GitHub creates the file; later saves include the returned sha |
| Close & Submit | `POST /repos/{o}/{r}/pulls` (head = branch, base = main) |
| Discard branch | `DELETE /repos/{o}/{r}/git/refs/heads/{branch}` |
| Detect stale sessions | `GET /repos/{o}/{r}/branches?protected=false` filter `docs/` + `GET /pulls?head=` |
| Sign in (first run) | `POST github.com/login/device/code`, then poll `POST github.com/login/oauth/access_token` |
| Validate sign-in / connect | `GET /user` and `GET /repos/{o}/{r}` (checks access) |

Note on `PUT /contents`: each save must send the *current* blob `sha` of the file **on the edit branch** (after the first save, use the sha returned by the previous save). This is what makes multiple commits per session work.

---

## 5. Configuration

### 5.1 `.env` template (ship this file as `.env.example`)

```bash
# ── Docs Editor configuration ──────────────────────────────
# Copy this file to `.env` in the same folder as the app,
# fill in your name, save. That's it.
#
# Signing in happens inside the app (Sign in with GitHub) —
# there are no tokens or passwords in this file.

# Your name as it should appear in edit history (e.g. "Ana")
DISPLAY_NAME=TODO_your_first_name

# ── Set by the team lead — do not change ───────────────────
# (example shown for the Cirdia documentation library; the writing
#  library uses REPO_OWNER=marycamacho / REPO_NAME=writing)
REPO_OWNER=cirdia-wellness
REPO_NAME=cirdia-documentation
DEFAULT_BRANCH=main

# Optional: more libraries for the in-app switcher
#REPOS=marycamacho/writing
```

**Auth decision (supersedes the PAT/keychain design):** sign-in is a **GitHub App device flow**. On launch the app connects with the stored sign-in; with none stored it shows the sign-in screen (name + "Sign in with GitHub" → short code → browser approval). The resulting non-expiring user token is written to an owner-only `auth.json` in the app data dir. `GITHUB_TOKEN` in `.env` survives as a dev/power-user override only — it is not part of any user's setup.

### 5.2 App config (baked in or `.env`)

`REPO_OWNER`, `REPO_NAME`, `DEFAULT_BRANCH`, optional `DOCS_ROOT` (e.g. only show the `/docs` folder), optional `BRANCH_PREFIX` (default `docs/`), optional `REPOS` (comma-separated additional `owner/repo[@branch]` libraries for the footer switcher).

Standard dotenv precedence: the `.env` file supplies values, and a real process env var overrides it. That's what the dev conveniences ride on — `npm run tauri:dev:docs` and `npm run tauri:dev:writing` launch against `cirdia-wellness/cirdia-documentation` and `marycamacho/writing` without touching the `.env`.

---

## 6. User Guide: Signing In (include verbatim in onboarding doc)

*The standalone, hand-to-a-team-member version of §6 + §6.1 (plus a short day-to-day section) is
maintained at [user-guide.md](user-guide.md) — that's the file to send people. Keep the two in
sync when either changes.*

> **Before you start:** you need a GitHub account, and the team lead must have given it access to the documents. If you can open the repo page your team lead sent you (e.g. github.com/cirdia-wellness/cirdia-documentation), you're set.
>
> 1. Open the Docs Editor app.
> 2. Type your name (this is how your edits are labeled — e.g. "Ana").
> 3. Click **Sign in with GitHub**.
> 4. The app shows a short code, something like `B4XR-9KQP`. Click the code to copy it.
> 5. Click **Open GitHub in your browser**. A GitHub page appears asking for the code.
> 6. Paste the code, click **Continue**, then click **Authorize**. (Sign in to github.com first if the browser asks you to.)
> 7. Switch back to the Docs Editor — your documents are already loading.
>
> That's it. **You stay signed in from now on** — quitting the app, restarting your computer, none of it signs you out. If your team uses more than one document library, this one sign-in covers all of them.
>
> **No passwords, no keys, nothing to save:** there is no token or password to keep track of. The sign-in lives safely on your computer, and it can be turned off any time from your GitHub account (Settings → Applications) or by the team lead.
>
> **If the browser says the code expired:** codes are only valid for a few minutes — go back to the app and click the sign-in button again for a fresh one.

*(Infra note for Mary: the "Cirdia Docs Editor" GitHub App — owned by @marycamacho, Client ID `Iv23liW7NEzKZeoZUVa0`, baked into `src-tauri/src/auth.rs` — has device flow enabled, user-token expiration disabled, Contents + Pull requests read/write, installable on Any account, no webhook, no private key. Installed on `cirdia-wellness/cirdia-documentation` and `marycamacho/writing`. Adding a library later = install the App on that repo. Kill switches: the user revokes under Settings → Applications, or you uninstall the App from the repo.)*

### 6.1 User Guide: Telling the App Which Document Library to Use (include after sign-in guide)

> **What you're doing:** pointing the Docs Editor at the team's document library. Takes 2 minutes, once.
>
> The app looks for a settings file called `.env` in the same folder as the app. Your team lead will send you a ready-made copy, or you can make one yourself:
>
> 1. Find the file **`.env.example`** in the app folder (it came with the app).
> 2. Make a copy of it and rename the copy to exactly **`.env`** — just those four characters, nothing before the dot.
>    - *Mac tip:* Finder may warn about names starting with a dot; click "Use ." to confirm.
>    - *Windows tip:* make sure it isn't secretly named `.env.txt` — turn on "File name extensions" in the File Explorer View menu to check.
> 3. Open `.env` in any text editor (TextEdit, Notepad — not Word).
> 4. Fill in your name where it says `DISPLAY_NAME=` (e.g. `DISPLAY_NAME=Ana`).
> 5. Check the repo lines match what your team lead told you:
>    ```
>    REPO_OWNER=cirdia-wellness
>    REPO_NAME=cirdia-documentation
>    DEFAULT_BRANCH=main
>    ```
>    These say *whose* library (`REPO_OWNER`), *which* library (`REPO_NAME`), and which version of it counts as the published one (`DEFAULT_BRANCH` — almost always `main`). If Mary sent you these values, copy them exactly. Don't guess.
> 6. Save the file and launch the app. If the footer of the app shows the repo name and your name, it worked.
>
> **If the app says it can't find the repo:** the three repo lines don't match a real repo, or you haven't been given access yet — ask your team lead to check both.
>
> **If your team lead sent you a pre-filled `.env`:** just drop it into the app folder, add your name on the `DISPLAY_NAME` line, done.

*(Note for Mary: the simplest rollout is to ship each person a pre-filled `.env` with everything but `DISPLAY_NAME` set — then their entire setup is: sign in + one name field.)*

---

## 7. Screens

1. **First run** — display name field + "Sign in with GitHub"; then the device-code step: big click-to-copy code, "Open GitHub in your browser" button, waiting state that polls until the browser approval lands (denied/expired return to the start with a plain message).
2. **Tree view** — sidebar of folders/files (only `.md`, only under `DOCS_ROOT`) with a **+ New document** button at top; main pane shows rendered preview of selected file; **Edit** button top-right. Small footer: connected repo + display name. When `REPOS` lists more than one library, the footer's repo name is a **click-to-switch menu**; switching reconnects with that library's own token (asking for it on first use) and is blocked mid-edit.
2a. **New document dialog** — Title field, folder dropdown (existing folders under `DOCS_ROOT` only — no new-folder creation in v1), live filename preview, duplicate warning, Create/Cancel.
3. **Editor** — CodeMirror source editing; toolbar: **Write / Split / Preview** view switch (HackMD-style; Split shows source and live rendered preview side by side; the choice persists across sessions), **Save**, **Close & Submit**, **Close without saving** (label switches to "Discard" and greys out after first save); dirty-state indicator; last-saved timestamp.
4. **Submitted** — checkmark, PR link, "Back to documents" button.
5. **Resume prompt** — appears on launch if unsubmitted `docs/*` branches by this user exist.

Design: follow the frontend-design skill / Cirdia app conventions; keep it to one accent color, system font stack is fine for v1.

---

## 8. Error Handling

| Failure | Behavior |
|---|---|
| Sign-in revoked (401) | Stored token is forgotten; route to the sign-in screen — signing in again fully recovers |
| No repo access (404 on repo) | "Ask Mary to add you to the docs repo" message |
| Network offline | Save button disabled with tooltip; editor buffer is preserved; retry banner |
| File changed on main during session (409 / sha mismatch on branch — shouldn't happen; on PR it becomes a normal conflict) | PR is still created; note in PR body "may need conflict resolution" |
| Rate limit | Show "GitHub is asking us to slow down — try again in a minute" |
| Save fails mid-session | Buffer never discarded; error toast with Retry |

Principle: the person's typed text is never lost. The buffer persists to local storage on every keystroke debounce (2s) so even a crash recovers.

---

## 9. Build Plan (suggested order)

1. Tauri scaffold + Svelte + env/config loading + token storage — ½ day
2. Device-flow sign-in + first-run screen — small
3. Tree fetch + render + file preview — ½ day
4. Editor pane (CodeMirror + preview) — ½ day
5. Branch-create / save / PR / discard flow with the API call map above — ½ day
5a. New-document dialog + create flow (reuses save/PR machinery) — small
6. Stale-session detection + resume — small
7. Error states, dirty-state guards, local buffer persistence — ½ day
8. Icon, build targets (macOS + Windows), `.env.example`, user guide doc — small

Roughly 2–3 focused days end to end.

---

## 10. Decisions (resolved 2026-08-13)

- [x] ~~**Token storage: both.** `.env` / keychain / per-repo keychain entries.~~ **Superseded 2026-08-13 (same day, after live testing):** the PAT + keychain design is replaced by **GitHub App device-flow sign-in** (§5.1, §6). Reasons: macOS keychain ACL password prompts (unfixable for unsigned dev builds), the PAT-creation burden on non-technical users, and fine-grained PATs being scoped to a single resource owner while the two libraries have different owners. The GitHub App gives one browser sign-in, non-expiring tokens (expiration disabled on the registration, matching the low-risk write-only-docs threat model), one sign-in covering all libraries, and a plain owner-only token file with zero OS prompts.
- [x] **Target repos:** `cirdia-wellness/cirdia-documentation` and `marycamacho/writing` — one repo per install via config. `DOCS_ROOT`: whole repo (both repos hold markdown throughout, not under a single `/docs` folder).
- [x] **Build targets:** Tauri desktop for both macOS and Windows.
- [x] **PR reviewer:** none requested by the app. Review assignment is left to repo defaults/CODEOWNERS.
- [x] **Token expiration: no expiration.** Now enforced structurally: user-token expiration is disabled on the GitHub App registration. Revocation (user's GitHub settings, or uninstalling the App) is the recovery path for a lost machine.