//! GitHub App device-flow sign-in and token storage.
//!
//! The person signs in once in their browser with a short code; GitHub issues
//! a non-expiring user token (expiration is disabled on the App registration).
//! The token lives in an owner-only file in the app's data directory — no
//! keychain, so no OS permission prompts, and the login survives quits and
//! reboots. One sign-in covers every library the App is installed on.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;

use crate::config;
use crate::github::ApiError;

const GITHUB: &str = "https://github.com";

/// The GitHub App's Client ID ("Cirdia Docs Editor", owned by @marycamacho) —
/// public information, not a secret (device flow uses no client secret).
/// Baked in so users configure nothing; the GITHUB_APP_CLIENT_ID config key
/// overrides it for testing against another App registration.
const DEFAULT_CLIENT_ID: &str = "Iv23liW7NEzKZeoZUVa0";

fn client_id() -> Result<String, ApiError> {
    config::resolve_all()
        .get("GITHUB_APP_CLIENT_ID")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| Some(DEFAULT_CLIENT_ID.to_string()).filter(|s| !s.is_empty()))
        .ok_or_else(|| {
            ApiError::state("The app build is missing its GitHub App Client ID — tell Mary.")
        })
}

// ── Pending device authorization ───────────────────────────────────────────

#[derive(Clone)]
pub struct Pending {
    device_code: String,
}

#[derive(Default)]
pub struct DeviceAuth(pub Mutex<Option<Pending>>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStart {
    pub user_code: String,
    pub verification_uri: String,
    /// Seconds the webview should wait between polls.
    pub interval: u64,
    pub expires_in: u64,
}

#[derive(Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    interval: Option<u64>,
    expires_in: Option<u64>,
}

/// Begin the sign-in: ask GitHub for a device code. The device code stays on
/// this side; the webview only sees what the person needs to read and type.
#[tauri::command]
pub async fn auth_start(state: State<'_, DeviceAuth>) -> Result<DeviceStart, ApiError> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{GITHUB}/login/device/code"))
        .header("Accept", "application/json")
        .header("User-Agent", "gh-document-editor")
        .form(&[("client_id", client_id()?)])
        .send()
        .await
        .map_err(ApiError::network)?;
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        return Err(ApiError::http(status, resp.text().await.unwrap_or_default()));
    }
    let body: DeviceCodeResponse = resp.json().await.map_err(ApiError::network)?;
    *state.0.lock().await = Some(Pending { device_code: body.device_code });
    Ok(DeviceStart {
        user_code: body.user_code,
        verification_uri: body.verification_uri,
        interval: body.interval.unwrap_or(5),
        expires_in: body.expires_in.unwrap_or(900),
    })
}

/// What one poll of GitHub's token endpoint means.
#[derive(Serialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PollOutcome {
    /// The person hasn't approved yet (or GitHub asked us to slow down).
    Pending,
    /// Approved — the token is saved; the webview should now connect.
    Connected,
    /// The code expired unused; start over.
    Expired,
    /// The person clicked Cancel in the browser.
    Denied,
}

/// Pure classification of GitHub's poll response, separated for tests.
pub fn classify_poll(body: &serde_json::Value) -> Result<PollOutcome, ApiError> {
    if body.get("access_token").and_then(|t| t.as_str()).is_some() {
        return Ok(PollOutcome::Connected);
    }
    match body.get("error").and_then(|e| e.as_str()) {
        Some("authorization_pending") | Some("slow_down") => Ok(PollOutcome::Pending),
        Some("expired_token") => Ok(PollOutcome::Expired),
        Some("access_denied") => Ok(PollOutcome::Denied),
        other => Err(ApiError::state(&format!(
            "Unexpected sign-in response from GitHub: {}",
            other.unwrap_or("no error field")
        ))),
    }
}

/// One poll: has the person approved in the browser yet? On approval the
/// token is written to disk before the webview hears "connected".
#[tauri::command]
pub async fn auth_poll(
    app: AppHandle,
    state: State<'_, DeviceAuth>,
) -> Result<PollOutcome, ApiError> {
    let pending = state
        .0
        .lock()
        .await
        .clone()
        .ok_or_else(|| ApiError::state("No sign-in in progress"))?;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{GITHUB}/login/oauth/access_token"))
        .header("Accept", "application/json")
        .header("User-Agent", "gh-document-editor")
        .form(&[
            ("client_id", client_id()?),
            ("device_code", pending.device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code".to_string()),
        ])
        .send()
        .await
        .map_err(ApiError::network)?;
    let body: serde_json::Value = resp.json().await.map_err(ApiError::network)?;
    let outcome = classify_poll(&body)?;
    if outcome == PollOutcome::Connected {
        let token = body["access_token"].as_str().unwrap_or_default().to_string();
        save_token_at(&token_path(&app)?, &token)?;
        *state.0.lock().await = None;
    }
    Ok(outcome)
}

/// Sign out: forget the stored token. (Revoking the app on github.com kills
/// the token server-side regardless.)
#[tauri::command]
pub fn auth_signout(app: AppHandle) -> Result<(), ApiError> {
    let path = token_path(&app)?;
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| ApiError::state(&format!("Couldn't remove the sign-in file: {e}")))?;
    }
    Ok(())
}

// ── Token file ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct TokenFile {
    access_token: String,
}

fn token_path(app: &AppHandle) -> Result<PathBuf, ApiError> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| ApiError::state(&format!("No app data directory: {e}")))?;
    Ok(dir.join("auth.json"))
}

pub fn stored_token(app: &AppHandle) -> Option<String> {
    load_token_at(&token_path(app).ok()?)
}

pub fn load_token_at(path: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(path).ok()?;
    let file: TokenFile = serde_json::from_str(&raw).ok()?;
    Some(file.access_token).filter(|t| !t.is_empty())
}

pub fn save_token_at(path: &Path, token: &str) -> Result<(), ApiError> {
    let err = |e: std::io::Error| ApiError::state(&format!("Couldn't save the sign-in: {e}"));
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(err)?;
    }
    let body = serde_json::to_string(&TokenFile { access_token: token.to_string() })
        .map_err(|e| ApiError::state(&format!("Couldn't save the sign-in: {e}")))?;
    std::fs::write(path, body).map_err(err)?;
    // Owner-only: the token file is private to this OS user.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(err)?;
    }
    Ok(())
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn classify_covers_the_documented_responses() {
        assert_eq!(classify_poll(&json!({"access_token": "gho_x"})).unwrap(), PollOutcome::Connected);
        assert_eq!(
            classify_poll(&json!({"error": "authorization_pending"})).unwrap(),
            PollOutcome::Pending
        );
        assert_eq!(classify_poll(&json!({"error": "slow_down"})).unwrap(), PollOutcome::Pending);
        assert_eq!(classify_poll(&json!({"error": "expired_token"})).unwrap(), PollOutcome::Expired);
        assert_eq!(classify_poll(&json!({"error": "access_denied"})).unwrap(), PollOutcome::Denied);
        assert!(classify_poll(&json!({"error": "incorrect_client_credentials"})).is_err());
    }

    #[test]
    fn token_file_round_trips_and_rejects_junk() {
        let dir = std::env::temp_dir().join(format!("docs-editor-test-{}", std::process::id()));
        let path = dir.join("auth.json");
        save_token_at(&path, "gho_test123").unwrap();
        assert_eq!(load_token_at(&path).unwrap(), "gho_test123");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }

        std::fs::write(&path, "{not json").unwrap();
        assert!(load_token_at(&path).is_none());
        std::fs::remove_dir_all(&dir).ok();
    }
}
