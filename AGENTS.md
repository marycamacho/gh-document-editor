# gh-document-editor — Agent Instructions

*Loaded when a session starts in this repo. It carries what you need for work here, and points to
the Cirdia documentation hub when a rule's full statement lives there.*

## What this repo is

A minimal **Tauri desktop app** that lets non-technical team members edit and create markdown docs
in a GitHub repo. Every edit lands on its own auto-created branch and arrives as a pull request;
the person never sees git. Svelte + CodeMirror front end; the Rust shell owns config resolution,
the keychain, and the GitHub REST client — all network traffic from the Rust layer, the ecosystem
pattern. No local clone, no git binary.

**The spec is the source of truth for behavior:** [docs/spec.md](docs/spec.md). When behavior
changes, the spec moves in the same change. The open decisions in spec §10 are Mary's calls — ask,
don't assume an answer.

**Two audiences, keep them separate in your head:**
- **Users** are non-technical team members. Everything they see — UI labels, error messages, the
  user-guide sections in spec §6 — is written in plain language for them. No git vocabulary in the
  UI: it's a "document library," you "save" and "submit for review."
- **Developers** (you) are held to the full Cirdia engineering bar. The users being non-technical
  raises the quality bar; it never lowers it.

The docs this tool edits live in repos like `cirdia-documentation` and `writing`. The tool moves
exactly the text the person typed — it never reformats, rewrites, or "fixes" document content.

## The bar (before anything else)

Be a genuine **expert** in this stack — Rust, Tauri 2, Svelte, TypeScript, CodeMirror, the GitHub
REST API — not a generalist reaching for the average answer. **Never code to the mediocre middle.**
Honor the **business requirement** over any convenient or off-the-shelf default, and if something
doesn't meet it, **say so** — don't quietly build on it. Full statement: the
[Cirdia rulebook](../cirdia-documentation/rulebook.md).

## Where it sits (so you calibrate the bar)

- **Org:** `marycamacho` for now; may move into the company later. Build it as a company-quality
  internal tool from day one — the org it lives in doesn't change the standard.
- **Lifecycle:** being built from the spec. Prototype-to-real transition applies (invariant #12):
  no CI to start, no hardening beyond what the stage needs, everything provisional is marked so.
- **Scale:** a small team. Don't build for load; GitHub's rate limits are the only volume concern.
- **Stakes:** medium and real. A team will use this daily, mostly on **macOS**, meaningfully on
  **Windows**. Two things carry the stakes: a person's typed text, and a PAT with write access to
  real repos.

## Product rules — load-bearing, from the spec

1. **Typed text is never lost.** The buffer persists to local storage on a keystroke debounce;
   errors never discard it; a crash recovers it. Any change that could drop a user's words is a
   bug regardless of what else it fixes.
2. **Auth is GitHub App device flow; the token is a secret the user never handles.** Sign-in is a
   browser approval with a short code — no PATs, no keychain (its ACL prompts are why it was
   removed). The non-expiring user token lives in an owner-only file in the app data dir, held by
   the Rust shell only — never logged, never in git, and **never in the webview** (the webview
   sees only the device code).
3. **All git operations go through the GitHub REST API, from the Rust layer.** No git2, no
   shell-outs, no local clone. The webview talks to GitHub only via the typed Tauri commands in
   `src-tauri/src/github.rs`.
4. **Errors speak the user's language.** Every failure state maps to a plain-language message with
   a next step (spec §8). A raw API error reaching the screen is a defect.
5. **Stay parameterized.** `owner`/`repo` come from config everywhere. Multi-repo is v2; hardcoding
   would make it a rewrite.
6. **Both platforms, every change.** Paths, keychain access, `.env` discovery, and filename
   handling work on macOS and Windows. Don't call a change done verified on only one.

## The rules that apply from the Cirdia system

The [rulebook's](../cirdia-documentation/rulebook.md) **engineering standards and working
agreement apply in full**. The ones that bite most often here:

- **Security, secrets, and infrastructure change only on an explicit directive.** That includes
  token storage, scopes, and anything about how the PAT is handled.
- **Tests come with the change**; run the suite before calling it done. All-pass means all-pass.
- **Docs move with the code** — here that means the spec and the user-guide text.
- **Config-as-code (#13):** non-secret config lives in committed files in one obvious place. The
  token is the one legitimate out-of-band value.
- **Verify before you claim**; a question you asked isn't answered until the human answers it.
- **No tech-debt punts** — fix what you hit on the path of your work.

**What does not apply:** the product-repo pipeline (tickets, tracks, trigger checks), the API
gateway, `app_id`, and the engine. This is tooling, like `scripts-automations` — it talks straight
to `api.github.com` by design, and GitHub is the platform this tool exists to front, not a
third-party exception to argue about.

## Working agreement

- **Git is Mary's.** Prepare the change and write the commit message and a copyable PR
  title/message; never commit, push, or merge unless she directs it.
- **Check in** where a question would get the real requirement instead of an assumption — the spec
  leaves decisions open on purpose.

## When to go up to the hub

For work inside this repo, this file and the spec are enough. Open the
[manifest](../cirdia-documentation/manifest.md) when work touches how Cirdia operates more broadly
(the doc repos this tool writes to, review conventions, or anything cross-repo).
