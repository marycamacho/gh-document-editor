use keyring::Entry;

/// GitHub tokens live in the OS keychain (macOS Keychain / Windows Credential
/// Manager) — never in localStorage, never in a file we write, and never
/// returned to the webview. Only the github module reads them.
///
/// Entries are keyed per repo (`owner/repo`): a fine-grained PAT is scoped to
/// a single resource owner, so different document libraries (e.g.
/// cirdia-wellness/cirdia-documentation vs marycamacho/writing) necessarily
/// carry different tokens. Per-repo entries let the app switch targets without
/// clobbering the other library's token.
const SERVICE: &str = "com.marycamacho.docs-editor";

fn entry(account: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, account).map_err(|e| e.to_string())
}

pub fn stored_token(account: &str) -> Option<String> {
    entry(account).ok()?.get_password().ok()
}

pub fn store_token(account: &str, token: &str) -> Result<(), String> {
    entry(account)?.set_password(token).map_err(|e| e.to_string())
}
