# Docs Editor

A small Tauri desktop app that lets non-technical team members edit and create markdown docs in a
GitHub repo. Every edit lands on its own auto-created branch and arrives as a pull request — the
person never sees git.

Full product and technical spec: [docs/spec.md](docs/spec.md). Setup guide to send team members:
[docs/user-guide.md](docs/user-guide.md). Agent working rules: [AGENTS.md](AGENTS.md).

## Stack

- **Tauri 2** desktop shell (macOS + Windows). Rust owns window management, `.env` config
  resolution, OS-keychain storage for the GitHub token, and the **GitHub REST client** (reqwest) —
  all network traffic runs in the Rust layer, and the token never enters the webview.
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

On first launch the app asks for a GitHub fine-grained PAT (Contents + Pull requests, read/write,
scoped to the one repo) and stores it in the OS keychain. Repo configuration comes from a `.env`
file — copy `.env.example` to `.env` next to the app (repo root in dev) and fill it in. A
`GITHUB_TOKEN` in `.env` skips the first-run screen entirely.

## Develop

```bash
npm run test:run    # frontend unit tests (Vitest)
npm run check       # svelte-check type checking
npm run test:all    # frontend tests + cargo test
npm run tauri:build # desktop bundles (macOS/Windows)
```

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
| `.env` loading + keychain | `src-tauri/src/config.rs`, `src-tauri/src/keychain.rs` |
