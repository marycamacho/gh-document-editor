use keyring::Entry;

/// The GitHub token lives in the OS keychain (macOS Keychain / Windows
/// Credential Manager) — never in localStorage, never in a file we write,
/// and never returned to the webview. Only the github module reads it.
const SERVICE: &str = "com.marycamacho.docs-editor";
const ACCOUNT: &str = "github-token";

fn entry() -> Result<Entry, String> {
    Entry::new(SERVICE, ACCOUNT).map_err(|e| e.to_string())
}

pub fn stored_token() -> Option<String> {
    entry().ok()?.get_password().ok()
}

pub fn store_token(token: &str) -> Result<(), String> {
    entry()?.set_password(token).map_err(|e| e.to_string())
}
