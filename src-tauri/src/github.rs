//! The GitHub REST client — all network traffic lives in the Rust layer,
//! matching the pattern of every app in the ecosystem. The webview invokes
//! typed commands; the PAT is held here and never returned to the webview.

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::{Method, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;
use tokio::sync::Mutex;

use crate::{config, keychain};

const API: &str = "https://api.github.com";

// ── Error contract with the webview ────────────────────────────────────────
// `kind` drives the frontend flow ("no-token" → first-run screen); `status`
// and `message` feed the plain-language error mapping in src/lib/errors.ts.

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub kind: String,
    pub status: Option<u16>,
    pub message: String,
}

impl ApiError {
    fn no_token() -> Self {
        Self { kind: "no-token".into(), status: None, message: "No stored token".into() }
    }
    fn network(e: impl std::fmt::Display) -> Self {
        Self { kind: "network".into(), status: None, message: e.to_string() }
    }
    fn http(status: u16, message: String) -> Self {
        Self { kind: "http".into(), status: Some(status), message }
    }
    fn state(message: &str) -> Self {
        Self { kind: "state".into(), status: None, message: message.into() }
    }
}

type ApiResult<T> = Result<T, ApiError>;

// ── Session state ──────────────────────────────────────────────────────────

#[derive(Clone)]
struct RepoCfg {
    owner: String,
    repo: String,
    default_branch: String,
}

#[derive(Clone)]
pub struct Session {
    token: String,
    login: String,
    cfg: RepoCfg,
    client: reqwest::Client,
}

#[derive(Default)]
pub struct Gh(pub Mutex<Option<Session>>);

async fn sess(state: &State<'_, Gh>) -> ApiResult<Session> {
    state
        .0
        .lock()
        .await
        .clone()
        .ok_or_else(|| ApiError::state("Not connected to GitHub yet"))
}

// ── HTTP plumbing ──────────────────────────────────────────────────────────

impl Session {
    fn req(&self, method: Method, path: &str) -> RequestBuilder {
        self.client
            .request(method, format!("{API}{path}"))
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("User-Agent", "gh-document-editor")
    }

    fn repo_path(&self, tail: &str) -> String {
        format!("/repos/{}/{}{}", self.cfg.owner, self.cfg.repo, tail)
    }
}

async fn send_json<T: for<'de> Deserialize<'de>>(rb: RequestBuilder) -> ApiResult<T> {
    let resp = rb.send().await.map_err(ApiError::network)?;
    let status = resp.status();
    if !status.is_success() {
        return Err(error_from(status, resp.text().await.unwrap_or_default()));
    }
    resp.json::<T>().await.map_err(ApiError::network)
}

async fn send_ok(rb: RequestBuilder) -> ApiResult<()> {
    let resp = rb.send().await.map_err(ApiError::network)?;
    let status = resp.status();
    if !status.is_success() {
        return Err(error_from(status, resp.text().await.unwrap_or_default()));
    }
    Ok(())
}

fn error_from(status: StatusCode, body: String) -> ApiError {
    let message = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(String::from))
        .unwrap_or(body);
    ApiError::http(status.as_u16(), message)
}

/// Percent-encode a repo path or branch name, keeping `/` separators.
fn enc(path: &str) -> String {
    path.split('/')
        .map(|seg| utf8_percent_encode(seg, NON_ALPHANUMERIC).to_string())
        .collect::<Vec<_>>()
        .join("/")
}

/// GitHub wraps base64 with newlines; strip all whitespace before decoding.
pub fn decode_content(b64: &str) -> ApiResult<String> {
    let compact: String = b64.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = B64
        .decode(compact)
        .map_err(|e| ApiError::state(&format!("Bad base64 from GitHub: {e}")))?;
    String::from_utf8(bytes).map_err(|e| ApiError::state(&format!("File is not UTF-8: {e}")))
}

pub fn encode_content(text: &str) -> String {
    B64.encode(text.as_bytes())
}

// ── Connect ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResult {
    pub login: String,
    /// False when the keychain refused to store a newly-entered token.
    pub stored: bool,
}

fn repo_cfg() -> ApiResult<RepoCfg> {
    let map = config::resolve_all();
    let get = |k: &str| map.get(k).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    Ok(RepoCfg {
        owner: get("REPO_OWNER").ok_or_else(|| ApiError::state("REPO_OWNER not configured"))?,
        repo: get("REPO_NAME").ok_or_else(|| ApiError::state("REPO_NAME not configured"))?,
        default_branch: get("DEFAULT_BRANCH").unwrap_or_else(|| "main".into()),
    })
}

/// The library to connect to: an explicit choice from the in-app switcher, or
/// the configured default when the webview passes nothing.
fn repo_cfg_with(
    owner: Option<String>,
    repo: Option<String>,
    default_branch: Option<String>,
) -> ApiResult<RepoCfg> {
    match (owner, repo) {
        (Some(owner), Some(repo)) => Ok(RepoCfg {
            owner,
            repo,
            default_branch: default_branch.unwrap_or_else(|| "main".into()),
        }),
        _ => repo_cfg(),
    }
}

#[derive(Deserialize)]
struct User {
    login: String,
}

/// The keychain account for a repo's token — one entry per document library,
/// because a fine-grained PAT is scoped to a single resource owner.
fn keychain_account(cfg: &RepoCfg) -> String {
    format!("{}/{}", cfg.owner, cfg.repo)
}

async fn validate_and_build(token: String, cfg: RepoCfg) -> ApiResult<Session> {
    let client = reqwest::Client::new();
    let mut session = Session { token, login: String::new(), cfg, client };
    let user: User = send_json(session.req(Method::GET, "/user")).await?;
    session.login = user.login;
    // Repo reachability — a bad repo name or missing access fails here as 404.
    send_ok(session.req(Method::GET, &session.repo_path(""))).await?;
    Ok(session)
}

/// Connect using the .env token or the keychain. Errors with kind "no-token"
/// when neither holds one — the frontend then shows the first-run screen.
#[tauri::command]
pub async fn session_connect(
    state: State<'_, Gh>,
    owner: Option<String>,
    repo: Option<String>,
    default_branch: Option<String>,
) -> ApiResult<ConnectResult> {
    let cfg = repo_cfg_with(owner, repo, default_branch)?;

    // Webview reloads (dev HMR, crash recovery) re-invoke this. Reuse the live
    // session instead of re-validating — and critically, instead of touching
    // the keychain again, which can prompt the person on macOS.
    if let Some(existing) = state.0.lock().await.clone() {
        if existing.cfg.owner == cfg.owner && existing.cfg.repo == cfg.repo {
            return Ok(ConnectResult { login: existing.login, stored: true });
        }
    }

    // Try the .env token first; the keychain read is deliberately lazy because
    // it can trigger a macOS permission prompt. An env token belongs to one
    // resource owner, so it legitimately fails against the other library —
    // the keychain entry is the per-library fallback.
    let env_token = config::resolve_all()
        .get("GITHUB_TOKEN")
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty() && !t.starts_with("TODO_"));
    let mut env_err: Option<ApiError> = None;
    if let Some(token) = env_token.clone() {
        match validate_and_build(token, cfg.clone()).await {
            Ok(session) => {
                let login = session.login.clone();
                *state.0.lock().await = Some(session);
                return Ok(ConnectResult { login, stored: true });
            }
            Err(e) => env_err = Some(e),
        }
    }

    let kc_token = keychain::stored_token(&keychain_account(&cfg)).filter(|t| {
        // Same token as .env already failed — don't validate it twice.
        env_token.as_deref() != Some(t.as_str())
    });
    if let Some(token) = kc_token {
        match validate_and_build(token, cfg.clone()).await {
            Ok(session) => {
                let login = session.login.clone();
                *state.0.lock().await = Some(session);
                return Ok(ConnectResult { login, stored: true });
            }
            // This library's own stored token failing (expired/revoked) is
            // worth surfacing as-is.
            Err(e) => return Err(e),
        }
    }

    // No keychain entry: a network-level env failure is worth reporting;
    // an auth/access failure of a foreign env token just means "ask for one".
    match env_err {
        Some(e) if e.kind == "network" => Err(e),
        _ => Err(ApiError::no_token()),
    }
}

/// First-run (once per library): validate a freshly-pasted token, keep it in
/// the keychain under this repo's entry, connect. The token comes in from the
/// webview once and is never sent back.
#[tauri::command]
pub async fn session_connect_token(
    state: State<'_, Gh>,
    token: String,
    owner: Option<String>,
    repo: Option<String>,
    default_branch: Option<String>,
) -> ApiResult<ConnectResult> {
    let cfg = repo_cfg_with(owner, repo, default_branch)?;
    let account = keychain_account(&cfg);
    let session = validate_and_build(token.trim().to_string(), cfg).await?;
    let stored = keychain::store_token(&account, &session.token).is_ok();
    let login = session.login.clone();
    *state.0.lock().await = Some(session);
    Ok(ConnectResult { login, stored })
}

// ── Repo operations (the API call map in docs/spec.md §4) ─────────────────

#[derive(Serialize)]
pub struct TreeResult {
    pub files: Vec<String>,
    pub truncated: bool,
}

#[derive(Deserialize)]
struct RefObj {
    object: RefSha,
}

#[derive(Deserialize)]
struct RefSha {
    sha: String,
}

#[derive(Deserialize)]
struct Tree {
    truncated: Option<bool>,
    tree: Vec<TreeEntry>,
}

#[derive(Deserialize)]
struct TreeEntry {
    path: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
}

async fn head_sha(s: &Session, branch: &str) -> ApiResult<String> {
    let r: RefObj = send_json(s.req(Method::GET, &s.repo_path(&format!("/git/ref/heads/{}", enc(branch))))).await?;
    Ok(r.object.sha)
}

#[tauri::command]
pub async fn gh_load_tree(state: State<'_, Gh>, docs_root: String) -> ApiResult<TreeResult> {
    let s = sess(&state).await?;
    let sha = head_sha(&s, &s.cfg.default_branch).await?;
    let tree: Tree =
        send_json(s.req(Method::GET, &s.repo_path(&format!("/git/trees/{sha}?recursive=1")))).await?;
    Ok(TreeResult {
        files: filter_md_paths(tree.tree.iter().filter_map(|e| match e.kind.as_deref() {
            Some("blob") => e.path.clone(),
            _ => None,
        }), &docs_root),
        truncated: tree.truncated.unwrap_or(false),
    })
}

/// Keep .md files, optionally only under the docs root. Pure, for tests.
pub fn filter_md_paths(paths: impl Iterator<Item = String>, docs_root: &str) -> Vec<String> {
    let prefix = if docs_root.is_empty() { String::new() } else { format!("{docs_root}/") };
    paths
        .filter(|p| p.to_lowercase().ends_with(".md") && (prefix.is_empty() || p.starts_with(&prefix)))
        .collect()
}

#[derive(Serialize)]
pub struct FileResult {
    pub content: String,
    pub sha: String,
}

#[derive(Deserialize)]
struct Contents {
    content: Option<String>,
    sha: String,
}

#[derive(Deserialize)]
struct Blob {
    content: String,
}

#[tauri::command]
pub async fn gh_read_file(
    state: State<'_, Gh>,
    path: String,
    git_ref: Option<String>,
) -> ApiResult<FileResult> {
    let s = sess(&state).await?;
    let r = git_ref.unwrap_or_else(|| s.cfg.default_branch.clone());
    let c: Contents = send_json(
        s.req(Method::GET, &s.repo_path(&format!("/contents/{}?ref={}", enc(&path), enc(&r)))),
    )
    .await?;
    let content = match c.content.as_deref() {
        Some(b64) if !b64.trim().is_empty() => decode_content(b64)?,
        // Files over ~1 MB come back without inline content — fetch the blob.
        _ => {
            let blob: Blob =
                send_json(s.req(Method::GET, &s.repo_path(&format!("/git/blobs/{}", c.sha)))).await?;
            decode_content(&blob.content)?
        }
    };
    Ok(FileResult { content, sha: c.sha })
}

/// Blob sha of a file at a ref; null when it doesn't exist there.
#[tauri::command]
pub async fn gh_file_sha(
    state: State<'_, Gh>,
    path: String,
    git_ref: String,
) -> ApiResult<Option<String>> {
    let s = sess(&state).await?;
    let result: ApiResult<Contents> = send_json(
        s.req(Method::GET, &s.repo_path(&format!("/contents/{}?ref={}", enc(&path), enc(&git_ref)))),
    )
    .await;
    match result {
        Ok(c) => Ok(Some(c.sha)),
        Err(e) if e.status == Some(404) => Ok(None),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn gh_create_branch(state: State<'_, Gh>, branch: String) -> ApiResult<()> {
    let s = sess(&state).await?;
    let sha = head_sha(&s, &s.cfg.default_branch).await?;
    send_ok(
        s.req(Method::POST, &s.repo_path("/git/refs"))
            .json(&json!({ "ref": format!("refs/heads/{branch}"), "sha": sha })),
    )
    .await
}

#[derive(Serialize)]
pub struct SaveResult {
    pub sha: String,
}

#[derive(Deserialize)]
struct SaveResponse {
    content: Contents,
}

#[tauri::command]
pub async fn gh_save_file(
    state: State<'_, Gh>,
    branch: String,
    path: String,
    content: String,
    message: String,
    sha: Option<String>,
) -> ApiResult<SaveResult> {
    let s = sess(&state).await?;
    let mut body = json!({
        "branch": branch,
        "message": message,
        "content": encode_content(&content),
    });
    if let Some(sha) = sha.filter(|v| !v.is_empty()) {
        body["sha"] = json!(sha);
    }
    let resp: SaveResponse =
        send_json(s.req(Method::PUT, &s.repo_path(&format!("/contents/{}", enc(&path)))).json(&body))
            .await?;
    Ok(SaveResult { sha: resp.content.sha })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrResult {
    pub number: u64,
    pub url: String,
}

#[derive(Deserialize)]
struct PrResponse {
    number: u64,
    html_url: String,
}

#[tauri::command]
pub async fn gh_create_pr(
    state: State<'_, Gh>,
    branch: String,
    title: String,
    body: String,
) -> ApiResult<PrResult> {
    let s = sess(&state).await?;
    let pr: PrResponse = send_json(s.req(Method::POST, &s.repo_path("/pulls")).json(&json!({
        "head": branch,
        "base": s.cfg.default_branch,
        "title": title,
        "body": body,
    })))
    .await?;
    Ok(PrResult { number: pr.number, url: pr.html_url })
}

/// Delete a session branch. Already-gone (404/422) is fine.
#[tauri::command]
pub async fn gh_delete_branch(state: State<'_, Gh>, branch: String) -> ApiResult<()> {
    let s = sess(&state).await?;
    match send_ok(s.req(Method::DELETE, &s.repo_path(&format!("/git/refs/heads/{}", enc(&branch))))).await
    {
        Ok(()) => Ok(()),
        Err(e) if matches!(e.status, Some(404) | Some(422)) => Ok(()),
        Err(e) => Err(e),
    }
}

// ── Stale-session detection (resume prompt) ────────────────────────────────

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaleSession {
    pub branch: String,
    pub path: Option<String>,
    pub commit_count: u64,
}

#[derive(Deserialize)]
struct BranchEntry {
    name: String,
}

#[derive(Deserialize)]
struct PrEntry {
    #[allow(dead_code)]
    number: u64,
}

#[derive(Deserialize)]
struct Compare {
    ahead_by: u64,
    commits: Vec<CompareCommit>,
    files: Option<Vec<CompareFile>>,
}

#[derive(Deserialize)]
struct CompareCommit {
    author: Option<User>,
}

#[derive(Deserialize)]
struct CompareFile {
    filename: Option<String>,
}

/// The pure decision: a branch is a resumable session of `login`'s when it has
/// commits ahead, no open PR, and one of its commits is theirs.
pub fn stale_from_compare(
    branch: &str,
    has_open_pr: bool,
    ahead_by: u64,
    commit_authors: &[Option<String>],
    login: &str,
    changed_files: &[String],
) -> Option<StaleSession> {
    if has_open_pr || ahead_by == 0 {
        return None;
    }
    if !commit_authors.iter().any(|a| a.as_deref() == Some(login)) {
        return None;
    }
    let path = changed_files.iter().find(|f| f.to_lowercase().ends_with(".md")).cloned();
    Some(StaleSession { branch: branch.to_string(), path, commit_count: ahead_by })
}

#[tauri::command]
pub async fn gh_find_stale_sessions(
    state: State<'_, Gh>,
    branch_prefix: String,
) -> ApiResult<Vec<StaleSession>> {
    let s = sess(&state).await?;
    let mut branches: Vec<BranchEntry> = Vec::new();
    for page in 1..=3 {
        let batch: Vec<BranchEntry> = send_json(
            s.req(Method::GET, &s.repo_path(&format!("/branches?per_page=100&page={page}"))),
        )
        .await?;
        let len = batch.len();
        branches.extend(batch);
        if len < 100 {
            break;
        }
    }

    let mut stale = Vec::new();
    for b in branches.iter().filter(|b| b.name.starts_with(&branch_prefix)) {
        let prs: Vec<PrEntry> = send_json(s.req(
            Method::GET,
            &s.repo_path(&format!("/pulls?state=open&head={}:{}", s.cfg.owner, enc(&b.name))),
        ))
        .await?;
        let cmp: Compare = send_json(s.req(
            Method::GET,
            &s.repo_path(&format!("/compare/{}...{}", enc(&s.cfg.default_branch), enc(&b.name))),
        ))
        .await?;
        let authors: Vec<Option<String>> =
            cmp.commits.iter().map(|c| c.author.as_ref().map(|a| a.login.clone())).collect();
        let files: Vec<String> = cmp
            .files
            .unwrap_or_default()
            .into_iter()
            .filter_map(|f| f.filename)
            .collect();
        if let Some(entry) =
            stale_from_compare(&b.name, !prs.is_empty(), cmp.ahead_by, &authors, &s.login, &files)
        {
            stale.push(entry);
        }
    }
    Ok(stale)
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trips_utf8() {
        let text = "# Café ✓ — naïve 🎉";
        assert_eq!(decode_content(&encode_content(text)).unwrap(), text);
    }

    #[test]
    fn base64_tolerates_githubs_newlines() {
        let b64 = encode_content("hello world");
        let wrapped: String = b64
            .chars()
            .enumerate()
            .flat_map(|(i, c)| if i > 0 && i % 8 == 0 { vec!['\n', c] } else { vec![c] })
            .collect();
        assert_eq!(decode_content(&wrapped).unwrap(), "hello world");
    }

    #[test]
    fn filter_keeps_only_md_under_root() {
        let paths = vec![
            "README.md".to_string(),
            "Company/guide.md".to_string(),
            "logo.png".to_string(),
            "docs/inner.md".to_string(),
        ];
        assert_eq!(
            filter_md_paths(paths.clone().into_iter(), ""),
            vec!["README.md", "Company/guide.md", "docs/inner.md"]
        );
        assert_eq!(filter_md_paths(paths.into_iter(), "docs"), vec!["docs/inner.md"]);
    }

    #[test]
    fn enc_keeps_separators_and_escapes_segments() {
        assert_eq!(enc("docs/my file.md"), "docs/my%20file%2Emd");
        assert_eq!(enc("docs/guide-20260813-1042"), "docs/guide%2D20260813%2D1042");
    }

    #[test]
    fn stale_requires_commits_no_pr_and_my_authorship() {
        let mine = vec![Some("ana".to_string())];
        let files = vec!["Company/guide.md".to_string(), "img.png".to_string()];

        let hit = stale_from_compare("docs/x", false, 2, &mine, "ana", &files).unwrap();
        assert_eq!(hit.path.as_deref(), Some("Company/guide.md"));
        assert_eq!(hit.commit_count, 2);

        assert!(stale_from_compare("docs/x", true, 2, &mine, "ana", &files).is_none());
        assert!(stale_from_compare("docs/x", false, 0, &mine, "ana", &files).is_none());
        let theirs = vec![Some("bob".to_string()), None];
        assert!(stale_from_compare("docs/x", false, 2, &theirs, "ana", &files).is_none());
    }

    #[test]
    fn stale_with_no_md_file_has_no_path() {
        let mine = vec![Some("ana".to_string())];
        let hit = stale_from_compare("docs/x", false, 1, &mine, "ana", &[]).unwrap();
        assert!(hit.path.is_none());
    }
}
