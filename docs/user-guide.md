# Docs Editor — Setup Guide

Welcome! The Docs Editor lets you read, edit, and create the team's documents. Your changes go in
for review automatically — no technical knowledge needed.

Setup is two short steps, about 4 minutes total, and you only do it once:

1. Sign in with your GitHub account
2. Tell the app which document library to use

---

## 1. Signing in

**Before you start:** you need a GitHub account, and the team lead must have given it access to
the documents. If you can open the repo page your team lead sent you (e.g.
github.com/cirdia-wellness/cirdia-documentation), you're set.

1. Open the Docs Editor app.
2. Type your name (this is how your edits are labeled — e.g. "Ana").
3. Click **Sign in with GitHub**.
4. The app shows a short code, something like `B4XR-9KQP`. Click the code to copy it.
5. Click **Open GitHub in your browser**. A GitHub page appears asking for the code.
6. Paste the code, click **Continue**, then click **Authorize**. (Sign in to github.com first if
   the browser asks you to.)
7. Switch back to the Docs Editor — your documents are already loading.

That's it. **You stay signed in from now on** — quitting the app, restarting your computer, none
of it signs you out. If your team uses more than one document library, this one sign-in covers
all of them.

**No passwords, no keys, nothing to save:** there is no token or password to keep track of. The
sign-in lives safely on your computer, and it can be turned off any time from your GitHub account
(Settings → Applications) or by the team lead.

**If the browser says the code expired:** codes are only valid for a few minutes — go back to the
app and click the sign-in button again for a fresh one.

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
  corner** to switch. Your sign-in already covers all of them.
- Your typed text is never lost: it's kept locally even if the app crashes or you go offline.

Questions or stuck? Ask Mary.
