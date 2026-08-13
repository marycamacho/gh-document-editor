# Docs Editor

A small Tauri desktop app that lets non-technical team members edit and create markdown docs in a
GitHub repo. Every edit lands on its own auto-created branch and arrives as a pull request — the
person never sees git.

Full product and technical spec: [docs/spec.md](docs/spec.md). Setup guide to send team members:
[docs/user-guide.md](docs/user-guide.md). Agent working rules: [AGENTS.md](AGENTS.md).

## Stack

- **Tauri 2** desktop shell (macOS + Windows). Rust owns window management, `.env` config
  resolution, sign-in + token storage, and the **GitHub REST client** (reqwest) — all network
  traffic runs in the Rust layer, and the token never enters the webview.
- **Auth:** GitHub App ("Cirdia Docs Editor") **device flow** — users approve once in the browser
  with a short code and stay signed in permanently (token expiration is disabled on the App).
  The token lives in an owner-only `auth.json` in the app data dir; no keychain, no OS prompts.
  One sign-in covers every library the App is installed on.
- **Svelte 5 + TypeScript + CodeMirror 6** front end in the webview, with HackMD-style
  Write / Split / Preview view modes. It talks to GitHub only through typed Tauri commands.
- **GitHub REST API** for everything git: branch, commit, PR. No local clone, no git binary.

## Run it locally

Prereqs: Node 20+, Rust stable, and on macOS the Xcode command line tools.

```bash
npm install
npm run tauri:dev            # uses .env (defaults to cirdia-documentation)
npm run tauri:dev:docs       # cirdia-wellness/cirdia-documentation
npm run tauri:dev:writing    # marycamacho/writing
```

On first launch the app shows "Sign in with GitHub": a short code, approved once in the browser.
Repo configuration comes from a `.env` file — copy `.env.example` to `.env` next to the app (repo
root in dev) and fill it in. For dev, a `GITHUB_TOKEN` in `.env` bypasses sign-in entirely, and
`GITHUB_APP_CLIENT_ID` overrides the baked-in App registration.

## Develop

```bash
npm run test:run    # frontend unit tests (Vitest)
npm run check       # svelte-check type checking
npm run test:all    # frontend tests + cargo test
npm run tauri:build # desktop bundles (macOS/Windows)
```

## Release

Releases are built by GitHub Actions ([.github/workflows/release.yml](.github/workflows/release.yml)):

1. Bump `version` in both `package.json` and `src-tauri/tauri.conf.json` (keep them equal), commit.
2. Run `npm run preflight` — checks version consistency and Tauri crate/npm alignment, and prints
   the exact fix if something's off. (CI runs the same script plus the full test suite in a
   `verify` job before any build starts.)
3. Tag and push: `git tag v0.1.0 && git push origin v0.1.0`.
4. The workflow builds macOS (Apple Silicon + Intel, signed + notarized) and Windows installers
   and attaches them to a **draft** GitHub Release — review it on the Releases page and publish.
5. Team members download from the repo's Releases page with their GitHub account.

macOS signing needs six repo secrets (listed at the top of the workflow file). To produce them:
export the **Developer ID Application** certificate from Keychain Access as a `.p12` with a
password, then `base64 -i cert.p12 | pbcopy` → `APPLE_CERTIFICATE`; the export password →
`APPLE_CERTIFICATE_PASSWORD`; the certificate's full name → `APPLE_SIGNING_IDENTITY`; your Apple
ID email, an app-specific password from appleid.apple.com, and the team ID → `APPLE_ID`,
`APPLE_PASSWORD`, `APPLE_TEAM_ID`. Windows builds are unsigned; users click through SmartScreen
("More info" → "Run anyway") on first launch.

The testable front-end logic lives in `src/lib/` (naming conventions, error mapping, tree
building, local drafts) with tests alongside. UI components are in `src/components/`. The Rust
side (`src-tauri/src/`) carries the GitHub client and has its own unit tests for the `.env`
parser, base64 handling, path encoding, and stale-session detection.

## Where things are

| Piece | Where |
|---|---|
| Branch/commit/PR naming (spec §3) | `src/lib/naming.ts` |
| GitHub API call map (spec §4) | `src-tauri/src/github.rs` (Rust client) + `src/lib/github.ts` (typed command wrappers) |
| Plain-language errors (spec §8) | `src/lib/errors.ts` |
| Draft persistence ("text is never lost") | `src/lib/localdb.ts` + autosave in `src/App.svelte` |
| Screens (spec §7) | `src/App.svelte` + `src/components/` |
| Sign-in (device flow) + token file | `src-tauri/src/auth.rs` |
| `.env` loading | `src-tauri/src/config.rs` |
