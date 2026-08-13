# Docs Editor — Setup Guide

Welcome! The Docs Editor lets you read, edit, and create the team's documents. Your changes go in
for review automatically — you never need to learn git or GitHub beyond the one-time setup below.

Setup is two short steps, about 5 minutes total, and you only do it once:

1. Create your personal access token (your "key" to the document library)
2. Tell the app which document library to use

---

## 1. Creating your GitHub token

**What you're doing:** creating a personal key that lets the Docs Editor save your changes to the
team's document library. Takes about 3 minutes. You do this once.

**Before you start:** you need a GitHub account and the team lead must have added you to the repo
with write access. If you can open the repo page your team lead sent you (e.g.
github.com/cirdia-wellness/cirdia-documentation), you're set.

**One token per document library.** A token only works for the one library it was created for. If
your team uses more than one library, you'll repeat these steps once per library — the app asks
for each one separately and remembers them all.

1. Go to **github.com** and sign in.
2. Click your **profile photo** (top-right corner) → **Settings**.
3. In the left sidebar, scroll to the bottom and click **Developer settings**.
4. Click **Personal access tokens** → **Fine-grained tokens**.
5. Click the green **Generate new token** button.
6. Fill in the form:
   - **Token name:** `docs-editor`
   - **Expiration:** choose **No expiration** (you do this once; if the token is ever lost or
     leaked, delete it and make a new one)
   - **Resource owner:** select the account your team lead told you (the organization or account
     that owns the document library — not your personal account, unless told otherwise)
   - **Repository access:** choose **Only select repositories**, then pick the repo your team lead
     told you (e.g. **cirdia-documentation**) from the dropdown
7. Under **Permissions → Repository permissions**, set exactly two:
   - **Contents:** Read and write
   - **Pull requests:** Read and write
   - Leave everything else on "No access."
8. Click **Generate token** at the bottom.
9. GitHub shows the token **once** — a long string starting with `github_pat_`. Click the
   **copy icon** next to it.
10. **Save it in Bitwarden before anything else.** Open Bitwarden, add a new Login item named
    `docs-editor` (put the token in the password field, and note which library it's for), and
    save. GitHub never shows the token again — Bitwarden is your backup copy.
11. Now open the Docs Editor app and paste the token when asked. It's stored safely in your
    computer's keychain. (Or, if your team lead set you up with the `.env` method: open the `.env`
    file next to the app, add a line `GITHUB_TOKEN=` followed by the token, save the file.)

**If you paste it wrong or the app loses it:** copy it again from Bitwarden. **If it's not in
Bitwarden either:** no problem — go back to step 4, delete the old token, and generate a new one.

**Never** paste the token into chat, email, or a shared doc. It's a password — Bitwarden and the
app are the only two places it should ever live.

---

## 2. Telling the app which document library to use

**What you're doing:** pointing the Docs Editor at the team's document library. Takes 2 minutes,
once.

The app looks for a settings file called `.env` in the same folder as the app. Your team lead will
send you a ready-made copy, or you can make one yourself:

1. Find the file **`.env.example`** in the app folder (it came with the app).
2. Make a copy of it and rename the copy to exactly **`.env`** — just those four characters,
   nothing before the dot.
   - *Mac tip:* Finder may warn about names starting with a dot; click "Use ." to confirm.
   - *Windows tip:* make sure it isn't secretly named `.env.txt` — turn on "File name extensions"
     in the File Explorer View menu to check.
3. Open `.env` in any text editor (TextEdit, Notepad — not Word).
4. Fill in your name where it says `DISPLAY_NAME=` (e.g. `DISPLAY_NAME=Ana`).
5. Check the repo lines match what your team lead told you:
   ```
   REPO_OWNER=cirdia-wellness
   REPO_NAME=cirdia-documentation
   DEFAULT_BRANCH=main
   ```
   These say *whose* library (`REPO_OWNER`), *which* library (`REPO_NAME`), and which version of it
   counts as the published one (`DEFAULT_BRANCH` — almost always `main`). If Mary sent you these
   values, copy them exactly. Don't guess.
6. Save the file and launch the app. If the footer of the app shows the repo name and your name,
   it worked.

**If the app says it can't find the repo:** the three repo lines don't match a real repo, or you
haven't been given access yet — ask your team lead to check both.

**If your team lead sent you a pre-filled `.env`:** just drop it into the app folder, add your name
on the `DISPLAY_NAME` line, done.

---

## Day to day

- Pick a document in the left sidebar to read it; click **Edit** to change it.
- **Save** as often as you like — every save is kept.
- The **Write / Split / Preview** buttons switch between the raw text, a side-by-side view, and
  the formatted result.
- When you're happy, click **Close & Submit** — your changes go in for review, and you'll get a
  link to follow along.
- Closed the app mid-edit? On next launch it offers to pick up where you left off.
- If your team uses more than one document library, click the **library name in the bottom-left
  corner** to switch. The first time you open a library it asks for that library's token (step 1
  above, once per library).
- Your typed text is never lost: it's kept locally even if the app crashes or you go offline.

Questions or stuck? Ask Mary.
