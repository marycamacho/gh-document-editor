// Pre-release checks — run locally before tagging (npm run preflight) and by
// the release workflow's verify job before any build starts. Every failure
// prints the exact fix; `npm run preflight -- --fix` applies the fixes it
// knows locally (never in CI — a tag pins a commit, so a fix is always a new
// commit).
import { readFileSync, writeFileSync } from "node:fs";
import { execSync } from "node:child_process";

const fixMode = process.argv.includes("--fix") && !process.env.GITHUB_REF;

let failed = false;
const ok = (msg) => console.log(`✓ ${msg}`);
const fail = (msg) => {
  console.error(`✗ ${msg}`);
  failed = true;
};
const fixed = (msg) => console.log(`⟳ fixed: ${msg}`);

const pkg = JSON.parse(readFileSync("package.json", "utf8"));
const conf = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));

// 1. App version consistent across package.json and tauri.conf.json.
//    package.json is the source of truth (npm version bumps it).
if (pkg.version !== conf.version) {
  if (fixMode) {
    conf.version = pkg.version;
    writeFileSync("src-tauri/tauri.conf.json", JSON.stringify(conf, null, 2) + "\n");
    fixed(`tauri.conf.json version → ${pkg.version}`);
  } else {
    fail(
      `app version mismatch: package.json ${pkg.version} vs tauri.conf.json ${conf.version} — set both to the same value (or run: npm run preflight -- --fix)`,
    );
  }
} else {
  ok(`app version ${pkg.version} consistent`);
}

// 2. Release tag (when running in CI on a tag) matches the app version.
const ref = process.env.GITHUB_REF ?? "";
if (ref.startsWith("refs/tags/v")) {
  const tagVersion = ref.slice("refs/tags/v".length);
  if (tagVersion !== pkg.version) {
    fail(
      `tag v${tagVersion} does not match app version ${pkg.version} — bump both version fields, commit, and re-tag`,
    );
  } else {
    ok(`tag v${tagVersion} matches app version`);
  }
}

// 3. Tauri Rust crate and @tauri-apps/api npm package on the same minor —
//    the Tauri CLI hard-fails the build otherwise.
const lock = readFileSync("src-tauri/Cargo.lock", "utf8");
const crateVersion = lock.match(/name = "tauri"\nversion = "([^"]+)"/)?.[1];
const apiVersion = JSON.parse(
  readFileSync("node_modules/@tauri-apps/api/package.json", "utf8"),
).version;
const minor = (v) => v.split(".").slice(0, 2).join(".");
if (!crateVersion) {
  fail("couldn't read the tauri crate version from src-tauri/Cargo.lock");
} else if (minor(crateVersion) !== minor(apiVersion)) {
  const cmd = `npm install @tauri-apps/api@^${minor(crateVersion)} @tauri-apps/cli@^${minor(crateVersion)}`;
  if (fixMode) {
    execSync(cmd, { stdio: "inherit" });
    fixed(`Tauri npm packages aligned to ${minor(crateVersion)} — review and commit`);
  } else {
    fail(
      `Tauri minor mismatch: tauri crate ${crateVersion} vs @tauri-apps/api ${apiVersion} — ` +
        `run: ${cmd} (or: npm run preflight -- --fix)`,
    );
  }
} else {
  ok(`tauri crate ${crateVersion} aligned with @tauri-apps/api ${apiVersion}`);
}

process.exit(failed ? 1 : 0);
