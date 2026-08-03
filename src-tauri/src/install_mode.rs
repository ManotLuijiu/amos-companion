//! Install mode detection for Linux
//! Detects whether Companion was installed via .deb package, AppImage, or tarball/script

use serde::{Deserialize, Serialize};

/// Represents how Companion is installed on the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallMode {
    /// Installation type: "deb" | "appimage" | "tarball"
    pub mode: String,
    /// Path to the current executable
    pub path: String,
}

/// Detect how Companion was installed on Linux
///
/// Detection order:
/// 1. AppImage - checked via APPIMAGE env var (set by AppImage runtime)
/// 2. .deb package - binary in system paths (/usr/bin, /usr/lib, /opt)
/// 3. Tarball/script - default, typically ~/.local/bin
pub fn detect_install_mode() -> InstallMode {
    // Check AppImage first
    // APPIMAGE env var is set by the AppImage runtime when executing
    if std::env::var("APPIMAGE").is_ok() {
        return InstallMode {
            mode: "appimage".to_string(),
            path: std::env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
        };
    }

    // Check system paths for .deb install
    if let Ok(exe_path) = std::env::current_exe() {
        let path_str = exe_path.to_string_lossy();

        // Common system package paths
        if path_str.contains("/usr/bin/")
            || path_str.contains("/usr/lib/")
            || path_str.contains("/opt/")
        {
            return InstallMode {
                mode: "deb".to_string(),
                path: path_str.to_string(),
            };
        }
    }

    // Default to tarball/script install
    InstallMode {
        mode: "tarball".to_string(),
        path: std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_install_mode_returns_valid_mode() {
        let mode = detect_install_mode();

        // Should return one of the valid modes
        assert!(
            mode.mode == "deb" || mode.mode == "appimage" || mode.mode == "tarball",
            "Invalid mode: {}",
            mode.mode
        );

        // Path should not be empty
        assert!(!mode.path.is_empty(), "Path should not be empty");
    }
}
