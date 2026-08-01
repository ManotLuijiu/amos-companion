//! ADB helper utilities — auto-installs ADB if not found.

use std::process::Command;
use tracing::{info, warn};

/// Return the bundled ADB path inside the companion data directory.
fn bundled_adb_path() -> std::path::PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("amos-companion")
        .join("platform-tools")
        .join(if cfg!(windows) { "adb.exe" } else { "adb" })
}

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
/// Falls back to auto-installing if not found anywhere.
pub fn find_adb() -> String {
    // 1. Bundled companion installation (takes priority — always up-to-date)
    let bundled = bundled_adb_path();
    if bundled.exists() {
        let path_str = bundled.to_string_lossy().to_string();
        info!("Found bundled adb at: {}", path_str);
        return path_str;
    }

    // 2. System / package-manager installations
    let candidates = [
        "/usr/local/bin/adb",    // Homebrew on Intel Mac
        "/opt/homebrew/bin/adb", // Homebrew on Apple Silicon
        "~/Library/Android/sdk/platform-tools/adb",
        "/usr/bin/adb",
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

/// Auto-install ADB using the dependency manager, then return the path.
/// Safe to call even if ADB is already present (install_adb is a no-op in that case).
pub fn install_adb_blocking() -> String {
    let bundled = bundled_adb_path();
    if bundled.exists() {
        return bundled.to_string_lossy().to_string();
    }

    // Try system install (apt / brew / choco) synchronously via tokio runtime
    warn!("ADB not found — attempting auto-install...");

    // Build a minimal runtime just for this call
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .ok();

    if let Some(rt) = rt {
        let result = rt.block_on(async {
            // Try apt/brew/choco first
            if crate::dependency_manager::try_system_install("adb").await.is_ok() {
                return Ok(());
            }
            // Fall back to bundled download
            crate::dependency_manager::install_adb().await
        });

        match result {
            Ok(()) => {
                info!("ADB auto-install succeeded");
            }
            Err(e) => {
                warn!("ADB auto-install failed: {}", e);
            }
        }
    }

    // Return whatever we can find now
    find_adb()
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
