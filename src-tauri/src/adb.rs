//! ADB helper utilities

use std::process::Command;
use tracing::{info, warn};

/// Expand ~ to home directory.
fn expand_tilde(path: &str) -> String {
    if path.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            return path.replace("~", home.to_str().unwrap_or(""));
        }
    }
    path.to_string()
}

/// Find the adb executable path. Tries multiple common locations.
pub fn find_adb() -> String {
    let candidates = [
        "/usr/local/bin/adb",    // Homebrew on Intel Mac
        "/opt/homebrew/bin/adb", // Homebrew on Apple Silicon
        "~/Library/Android/sdk/platform-tools/adb",
        "/usr/bin/adb",
        "adb", // Fallback to PATH
    ];

    for candidate in &candidates {
        let path = expand_tilde(candidate);
        if std::path::Path::new(&path).exists() {
            info!("Found adb at: {}", path);
            return path;
        }
    }

    warn!("adb not found in common locations, trying PATH");
    "adb".to_string()
}

/// Run an ADB command and return stdout.
pub fn run_adb(args: &[&str]) -> Result<String, String> {
    let adb_path = find_adb();
    let output = Command::new(&adb_path)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run adb ({}): {}", adb_path, e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("ADB command failed: {}", stderr))
    }
}
