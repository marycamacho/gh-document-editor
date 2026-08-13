use std::collections::HashMap;
use std::path::PathBuf;

/// Baked-in defaults: the libraries this app exists to edit. The GitHub App
/// ("Cirdia Docs Editor") is installed on exactly these repos, so they are a
/// fact about the product, not per-user configuration — a fresh install works
/// with no .env at all (config-as-code). A .env or process env var overrides
/// them for dev or unusual setups.
const DEFAULTS: [(&str, &str); 4] = [
    ("REPO_OWNER", "cirdia-wellness"),
    ("REPO_NAME", "cirdia-documentation"),
    ("DEFAULT_BRANCH", "main"),
    ("REPOS", "marycamacho/writing"),
];

/// The configuration keys the app understands. Only these are read from the
/// environment and .env — nothing else leaks into the webview.
const KEYS: [&str; 9] = [
    "GITHUB_TOKEN",
    "GITHUB_APP_CLIENT_ID",
    "DISPLAY_NAME",
    "REPO_OWNER",
    "REPO_NAME",
    "DEFAULT_BRANCH",
    "DOCS_ROOT",
    "BRANCH_PREFIX",
    "REPOS",
];

/// Parse simple KEY=VALUE lines. Supports comments, blank lines, CRLF, and
/// optional single/double quotes around the value. No interpolation.
pub fn parse_env(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        let mut value = value.trim();
        if value.len() >= 2
            && ((value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\'')))
        {
            value = &value[1..value.len() - 1];
        }
        map.insert(key.to_string(), value.to_string());
    }
    map
}

/// Where a `.env` may live, in order of preference:
/// 1. the working directory (dev),
/// 2. next to the executable (Windows install folder),
/// 3. next to the .app bundle (macOS: exe is App.app/Contents/MacOS/exe).
fn candidate_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join(".env"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            paths.push(exe_dir.join(".env"));
            #[cfg(target_os = "macos")]
            if let Some(bundle_parent) = exe_dir.ancestors().nth(3) {
                paths.push(bundle_parent.join(".env"));
            }
        }
    }
    paths
}

/// Full resolved configuration, token included — internal use only (the
/// github module reads GITHUB_TOKEN from here). Standard dotenv precedence:
/// the `.env` file provides values, a real process env var overrides it —
/// which is what lets `REPO_OWNER=x REPO_NAME=y npm run tauri:dev` target a
/// different repo without touching the file.
pub fn resolve_all() -> HashMap<String, String> {
    let mut map: HashMap<String, String> =
        DEFAULTS.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    for path in candidate_paths() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            for (key, value) in parse_env(&content) {
                // Empty override values never blank out a baked default.
                if KEYS.contains(&key.as_str()) && !value.trim().is_empty() {
                    map.insert(key, value);
                }
            }
            break;
        }
    }
    for key in KEYS {
        if let Ok(value) = std::env::var(key) {
            if !value.trim().is_empty() {
                map.insert(key.to_string(), value);
            }
        }
    }
    map
}

/// Config for the webview. The token is stripped — it never crosses into the
/// webview; the Rust github client is the only thing that holds it.
#[tauri::command]
pub fn load_config() -> HashMap<String, String> {
    let mut map = resolve_all();
    map.remove("GITHUB_TOKEN");
    map
}

#[cfg(test)]
mod tests {
    use super::parse_env;

    #[test]
    fn parses_plain_pairs() {
        let map = parse_env("REPO_OWNER=cirdia-wellness\nREPO_NAME=cirdia-documentation\n");
        assert_eq!(map["REPO_OWNER"], "cirdia-wellness");
        assert_eq!(map["REPO_NAME"], "cirdia-documentation");
    }

    #[test]
    fn skips_comments_and_blanks() {
        let map = parse_env("# comment\n\n  \nDISPLAY_NAME=Ana\n#DISPLAY_NAME=Bob\n");
        assert_eq!(map.len(), 1);
        assert_eq!(map["DISPLAY_NAME"], "Ana");
    }

    #[test]
    fn trims_whitespace_and_strips_quotes() {
        let map = parse_env("  DISPLAY_NAME =  \"Ana Maria\" \nREPO_OWNER='cirdia-wellness'\n");
        assert_eq!(map["DISPLAY_NAME"], "Ana Maria");
        assert_eq!(map["REPO_OWNER"], "cirdia-wellness");
    }

    #[test]
    fn handles_crlf() {
        let map = parse_env("REPO_OWNER=x\r\nREPO_NAME=y\r\n");
        assert_eq!(map["REPO_OWNER"], "x");
        assert_eq!(map["REPO_NAME"], "y");
    }

    #[test]
    fn keeps_equals_signs_in_values() {
        let map = parse_env("GITHUB_TOKEN=github_pat_a=b=c\n");
        assert_eq!(map["GITHUB_TOKEN"], "github_pat_a=b=c");
    }

    #[test]
    fn ignores_lines_without_equals_or_key() {
        let map = parse_env("not a pair\n=orphan-value\nDISPLAY_NAME=Ana\n");
        assert_eq!(map.len(), 1);
    }
}
