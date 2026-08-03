//! Dependency Manager - Auto-installs Node.js, scrcpy, ffmpeg, and ws-scrcpy
//!
//! This module handles automatic installation of all dependencies needed for
//! ws-scrcpy mirroring. Everything is installed to the companion's data directory.

use std::path::PathBuf;
use std::process::Command;
use tracing::{info, warn};

// ─── Version Constants ─────────────────────────────────────────────────────────

/// ws-scrcpy GitHub repo
const WS_SCRCPY_REPO: &str = "https://github.com/NetrisTV/ws-scrcpy.git";

// ─── Directory Helpers ────────────────────────────────────────────────────────

/// Base directory for all companion dependencies
pub fn get_companion_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("amos-companion")
}

/// Directory for Node.js installation
pub fn get_nodejs_dir() -> PathBuf {
    get_companion_dir().join("nodejs")
}

/// Directory for ws-scrcpy
pub fn get_ws_scrcpy_dir() -> PathBuf {
    get_companion_dir().join("ws-scrcpy")
}

/// Directory for scrcpy binary
pub fn get_scrcpy_dir() -> PathBuf {
    get_companion_dir().join("scrcpy")
}

/// Directory for ffmpeg
pub fn get_ffmpeg_dir() -> PathBuf {
    get_companion_dir().join("ffmpeg")
}

/// Get the Node.js binary path
pub fn get_nodejs_bin() -> PathBuf {
    let nodejs_dir = get_nodejs_dir();
    let exe = if cfg!(target_os = "windows") { "node.exe" } else { "node" };
    nodejs_dir.join("bin").join(exe)
}

/// Get the npm binary path
pub fn get_npm_bin() -> PathBuf {
    let nodejs_dir = get_nodejs_dir();
    let exe = if cfg!(target_os = "windows") { "npm.cmd" } else { "npm" };
    nodejs_dir.join("bin").join(exe)
}

/// Get the scrcpy binary path
pub fn get_scrcpy_bin() -> PathBuf {
    let scrcpy_dir = get_scrcpy_dir();
    let exe = if cfg!(target_os = "windows") { "scrcpy.exe" } else { "scrcpy" };
    let bin_path = scrcpy_dir.join("bin").join(exe);
    if bin_path.exists() {
        return bin_path;
    }
    let alt_path = scrcpy_dir.join("scrcpy").join(exe);
    if alt_path.exists() {
        return alt_path;
    }
    scrcpy_dir.join(exe)
}

/// Get the ffmpeg binary path
pub fn get_ffmpeg_bin() -> PathBuf {
    let ffmpeg_dir = get_ffmpeg_dir();
    let exe = if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" };
    ffmpeg_dir.join("bin").join(exe)
}

/// Directory for ADB (Android SDK Platform Tools)
pub fn get_adb_dir() -> PathBuf {
    get_companion_dir().join("platform-tools")
}

/// Get the adb binary path
pub fn get_adb_bin() -> PathBuf {
    let adb_dir = get_adb_dir();
    let exe = if cfg!(target_os = "windows") { "adb.exe" } else { "adb" };
    adb_dir.join(exe)
}

// ─── OS Detection ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OS {
    Linux,
    Macos,
    Windows,
}

impl OS {
    pub fn current() -> Self {
        match std::env::consts::OS {
            "linux" => OS::Linux,
            "macos" => OS::Macos,
            "windows" => OS::Windows,
            _ => panic!("Unsupported OS"),
        }
    }
}

impl std::fmt::Display for OS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OS::Linux => write!(f, "linux"),
            OS::Macos => write!(f, "macos"),
            OS::Windows => write!(f, "windows"),
        }
    }
}

// ─── Download URLs ─────────────────────────────────────────────────────────────

fn get_nodejs_url(os: OS) -> &'static str {
    match os {
        OS::Linux => "https://nodejs.org/dist/v20.10.0/node-v20.10.0-linux-x64.tar.xz",
        OS::Macos => {
            if std::env::consts::ARCH == "aarch64" {
                "https://nodejs.org/dist/v20.10.0/node-v20.10.0-darwin-arm64.tar.gz"
            } else {
                "https://nodejs.org/dist/v20.10.0/node-v20.10.0-darwin-x64.tar.gz"
            }
        }
        OS::Windows => "https://nodejs.org/dist/v20.10.0/node-v20.10.0-win-x64.zip",
    }
}

fn get_scrcpy_url(os: OS) -> &'static str {
    match os {
        OS::Linux => "https://github.com/Genymobile/scrcpy/releases/download/v2.0/scrcpy-linux-x86_64.zip",
        OS::Macos => "https://github.com/Genymobile/scrcpy/releases/download/v2.0/scrcpy-macos.zip",
        OS::Windows => "https://github.com/Genymobile/scrcpy/releases/download/v2.0/scrcpy-win64.zip",
    }
}

fn get_ffmpeg_url(os: OS) -> &'static str {
    match os {
        OS::Linux => "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz",
        OS::Macos => "https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip",
        OS::Windows => "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip",
    }
}

fn get_adb_url(os: OS) -> &'static str {
    // Android SDK Platform Tools — latest release
    match os {
        OS::Linux   => "https://dl.google.com/android/repository/platform-tools-latest-linux.zip",
        OS::Macos   => "https://dl.google.com/android/repository/platform-tools-latest-darwin.zip",
        OS::Windows => "https://dl.google.com/android/repository/platform-tools-latest-windows.zip",
    }
}

// ─── Installation Status ────────────────────────────────────────────────────────

/// Check if a binary exists at a specific path
fn check_path(path: &str) -> bool {
    PathBuf::from(path).exists()
}

/// Check if Node.js is installed (bundled or system)
pub fn is_nodejs_installed() -> bool {
    // Check common system paths first
    if check_path("/usr/local/bin/node") || check_path("/usr/bin/node") || check_path("/opt/homebrew/bin/node") {
        info!("Found Node.js at standard system path");
        return true;
    }

    // Check nvm managed nodes (common on Linux and macOS)
    if let Ok(home) = std::env::var("HOME") {
        // Check common nvm version paths
        let nvm_versions = ["v24.18.0", "v24.18.1", "v22.0.0", "v20.0.0", "v18.0.0"];
        for version in &nvm_versions {
            let nvm_path = format!("{}/.nvm/versions/node/{}/bin/node", home, version);
            if check_path(&nvm_path) {
                info!("Found Node.js at nvm path: {}", nvm_path);
                return true;
            }
        }
        // Also check if ~/.local/bin/node exists (other node managers)
        let local_bin_node = format!("{}/.local/bin/node", home);
        if check_path(&local_bin_node) {
            info!("Found Node.js at local bin path: {}", local_bin_node);
            return true;
        }
    }

    // Check system node via which (works on Linux/Windows)
    if let Ok(output) = Command::new("which").arg("node").output() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() && PathBuf::from(&path).exists() {
            info!("Found system node at: {}", path);
            return true;
        }
    }

    // Check bundled node
    get_nodejs_bin().exists()
}

/// Check if scrcpy is installed (either system or bundled)
pub fn is_scrcpy_installed() -> bool {
    // Check known macOS paths first
    if cfg!(target_os = "macos")
        && (check_path("/usr/local/bin/scrcpy") || check_path("/opt/homebrew/bin/scrcpy")) {
            info!("Found scrcpy at standard macOS path");
            return true;
        }

    if let Ok(output) = Command::new("which").arg("scrcpy").output() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() && PathBuf::from(&path).exists() {
            info!("Found system scrcpy at: {}", path);
            return true;
        }
    }
    get_scrcpy_bin().exists()
}

/// Check if ffmpeg is installed (either system or bundled)
pub fn is_ffmpeg_installed() -> bool {
    // Check known macOS paths first
    if cfg!(target_os = "macos")
        && (check_path("/usr/local/bin/ffmpeg") || check_path("/opt/homebrew/bin/ffmpeg") || check_path("/usr/bin/ffmpeg")) {
            info!("Found ffmpeg at standard macOS path");
            return true;
        }

    if let Ok(output) = Command::new("which").arg("ffmpeg").output() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() && PathBuf::from(&path).exists() {
            info!("Found system ffmpeg at: {}", path);
            return true;
        }
    }
    get_ffmpeg_bin().exists()
}

/// Check if ws-scrcpy is installed
pub fn is_ws_scrcpy_installed() -> bool {
    let ws_dir = get_ws_scrcpy_dir();
    ws_dir.exists() && ws_dir.join("package.json").exists()
}

/// Check if ADB is installed (either system or bundled)
pub fn is_adb_installed() -> bool {
    // Check known macOS paths first
    if cfg!(target_os = "macos")
        && (check_path("/usr/local/bin/adb") || check_path("/opt/homebrew/bin/adb") || check_path("/usr/bin/adb")) {
            info!("Found ADB at standard macOS path");
            return true;
        }

    if let Ok(output) = Command::new("which").arg("adb").output() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() && PathBuf::from(&path).exists() {
            info!("Found system adb at: {}", path);
            return true;
        }
    }
    get_adb_bin().exists()
}

/// Check if all dependencies are installed
pub fn are_all_deps_installed() -> bool {
    is_nodejs_installed() && is_scrcpy_installed() && is_ffmpeg_installed() && is_ws_scrcpy_installed() && is_adb_installed()
}

// ─── System Installation ────────────────────────────────────────────────────────

/// Try system installation (apt, brew, etc.)
pub async fn try_system_install(package: &str) -> Result<(), String> {
    let os = OS::current();

    match os {
        OS::Linux => {
            info!("Trying system install via apt: {}", package);
            // Try pkexec first (graphical apps), then sudo
            for cmd in &["pkexec", "sudo"] {
                let output = Command::new(cmd)
                    .args(["apt", "install", "-y", package])
                    .output();

                match output {
                    Ok(out) if out.status.success() => {
                        info!("Successfully installed {} via {} apt", package, cmd);
                        return Ok(());
                    }
                    Ok(out) => {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        warn!("{} apt install {} failed: {}", cmd, package, stderr);
                        // Try next method
                    }
                    Err(e) => {
                        warn!("Failed to run {} apt: {}", cmd, e);
                    }
                }
            }
            Err(format!("Could not install {} via apt (tried pkexec, sudo)", package))
        }
        OS::Macos => {
            info!("Trying system install via brew: {}", package);
            let output = Command::new("brew")
                .args(["install", package])
                .output();

            match output {
                Ok(out) if out.status.success() => {
                    info!("Successfully installed {} via brew", package);
                    Ok(())
                }
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    warn!("brew install {} failed: {}", package, stderr);
                    Err(format!("brew failed: {}", stderr))
                }
                Err(e) => Err(format!("Failed to run brew: {}", e)),
            }
        }
        OS::Windows => {
            info!("Trying system install via choco: {}", package);
            let output = Command::new("choco")
                .args(["install", package, "-y"])
                .output();

            match output {
                Ok(out) if out.status.success() => {
                    info!("Successfully installed {} via choco", package);
                    Ok(())
                }
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    warn!("choco install {} failed: {}", package, stderr);
                    Err(format!("choco failed: {}", stderr))
                }
                Err(e) => Err(format!("Failed to run choco: {}", e)),
            }
        }
    }
}

// ─── Download Helper ───────────────────────────────────────────────────────────

/// Download file using wget (better redirect following than curl for CDN URLs)
async fn download_file(url: &str, dest: &PathBuf) -> Result<(), String> {
    info!("Downloading from: {}", url);

    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // Use wget for better redirect following (GitHub CDN redirects work reliably)
    let output = Command::new("wget")
        .args(["-q", "-O", &dest.to_string_lossy(), url])
        .output()
        .map_err(|e| format!("Failed to run wget: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!("wget failed (exit {}): {} {}",
            output.status.code().unwrap_or(-1), stderr, stdout));
    }

    // Verify we got a real file (not HTML redirect page)
    let file_size = std::fs::metadata(dest)
        .map(|m| m.len())
        .unwrap_or(0);
    if file_size < 1000 {
        // Very small file — likely HTML error or redirect page
        let content = std::fs::read_to_string(dest)
            .unwrap_or_default();
        if content.contains("<html") || content.contains("<!DOCTYPE") {
            return Err(format!("Download returned HTML instead of binary ({} bytes). URL may be invalid or rate-limited.", file_size));
        }
    }

    info!("Downloaded {} bytes to {:?}", file_size, dest);
    Ok(())
}

/// Extract archive using system tools
fn extract_archive(archive: &PathBuf, dest_dir: &PathBuf, is_zip: bool) -> Result<(), String> {
    info!("Extracting {:?} to {:?}", archive, dest_dir);

    std::fs::create_dir_all(dest_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    let output = if is_zip {
        Command::new("unzip")
            .args(["-o", &archive.to_string_lossy(), "-d", &dest_dir.to_string_lossy()])
            .output()
    } else {
        // For tar.xz, use tar
        Command::new("tar")
            .args(["-xf", &archive.to_string_lossy(), "-C", &dest_dir.to_string_lossy()])
            .output()
    };

    match output {
        Ok(out) if out.status.success() => {
            info!("Extraction successful");
            Ok(())
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            Err(format!("Extraction failed: {}", stderr))
        }
        Err(e) => Err(format!("Failed to run extraction command: {}", e)),
    }
}

// ─── Install Functions ────────────────────────────────────────────────────────

/// Make binaries executable (Unix only)
#[cfg(unix)]
fn make_executable(dir: &PathBuf) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(mut perms) = std::fs::metadata(&path).map(|m| m.permissions()) {
                    perms.set_mode(0o755);
                    let _ = std::fs::set_permissions(&path, perms);
                }
            }
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_dir: &PathBuf) -> Result<(), String> {
    Ok(())
}

/// Install Node.js (portable bundle)
pub async fn install_nodejs() -> Result<(), String> {
    if is_nodejs_installed() {
        info!("Node.js already installed");
        return Ok(());
    }

    info!("Installing Node.js v20.10.0...");

    let os = OS::current();
    let url = get_nodejs_url(os);
    let dest_dir = get_nodejs_dir();
    
    // Use temp file for download
    let temp_dir = get_companion_dir().join("temp");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;
    
    let temp_file = temp_dir.join("nodejs_archive");
    let is_zip = url.ends_with(".zip");

    download_file(url, &temp_file).await?;
    extract_archive(&temp_file, &dest_dir, is_zip)?;
    
    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);

    make_executable(&dest_dir.join("bin"))?;

    info!("Node.js installed successfully");
    Ok(())
}

/// Install scrcpy
pub async fn install_scrcpy() -> Result<(), String> {
    if is_scrcpy_installed() {
        info!("scrcpy already installed");
        return Ok(());
    }

    // Try system install first
    if try_system_install("scrcpy").await.is_ok() {
        info!("scrcpy installed via system package manager");
        return Ok(());
    }

    info!("Installing scrcpy (prebuilt binary)...");

    let os = OS::current();
    let url = get_scrcpy_url(os);
    let dest_dir = get_scrcpy_dir();
    
    let temp_dir = get_companion_dir().join("temp");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;
    
    let temp_file = temp_dir.join("scrcpy_archive.zip");

    download_file(url, &temp_file).await?;
    extract_archive(&temp_file, &dest_dir, true)?;
    
    let _ = std::fs::remove_file(&temp_file);

    make_executable(&dest_dir)?;

    info!("scrcpy installed successfully");
    Ok(())
}

/// Install ffmpeg
pub async fn install_ffmpeg() -> Result<(), String> {
    if is_ffmpeg_installed() {
        info!("ffmpeg already installed");
        return Ok(());
    }

    // Try system install first
    if try_system_install("ffmpeg").await.is_ok() {
        info!("ffmpeg installed via system package manager");
        return Ok(());
    }

    info!("Installing ffmpeg (prebuilt binary)...");

    let os = OS::current();
    let url = get_ffmpeg_url(os);
    let dest_dir = get_ffmpeg_dir();
    
    let temp_dir = get_companion_dir().join("temp");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;
    
    let is_zip = url.ends_with(".zip");
    let ext = if is_zip { "zip" } else { "tar.xz" };
    let temp_file = temp_dir.join(format!("ffmpeg_archive.{}", ext));

    download_file(url, &temp_file).await?;
    extract_archive(&temp_file, &dest_dir, is_zip)?;
    
    let _ = std::fs::remove_file(&temp_file);

    make_executable(&dest_dir.join("bin"))?;

    info!("ffmpeg installed successfully");
    Ok(())
}

/// Clone or update ws-scrcpy repository
pub async fn install_ws_scrcpy() -> Result<(), String> {
    let ws_dir = get_ws_scrcpy_dir();

    if ws_dir.exists() {
        info!("ws-scrcpy directory exists, updating...");

        let output = Command::new("git")
            .args(["pull", "origin", "main"])
            .current_dir(&ws_dir)
            .output()
            .map_err(|e| format!("Failed to run git pull: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("git pull failed (may be on different branch): {}", stderr);
        }
    } else {
        info!("Cloning ws-scrcpy from {}", WS_SCRCPY_REPO);

        if let Some(parent) = ws_dir.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let output = Command::new("git")
            .args(["clone", WS_SCRCPY_REPO, ws_dir.to_str().unwrap_or("")])
            .output()
            .map_err(|e| format!("Failed to run git clone: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Git clone failed: {}", stderr));
        }
    }

    // Install npm dependencies
    let npm = get_npm_bin();
    if !npm.exists() {
        return Err(format!("npm not found at {:?}", npm));
    }

    info!("Installing ws-scrcpy npm dependencies...");

    let output = Command::new(&npm)
        .args(["install", "--production=false"])
        .current_dir(&ws_dir)
        .env("PATH", get_path_env())
        .output()
        .map_err(|e| format!("Failed to run npm install: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("npm install failed: {}", stderr));
    }

    info!("ws-scrcpy installed successfully");
    Ok(())
}

/// Get PATH environment with companion binaries
pub fn get_path_env() -> String {
    let mut paths = Vec::new();

    let nodejs_bin = get_nodejs_dir().join("bin");
    if nodejs_bin.exists() {
        paths.push(nodejs_bin.to_string_lossy().to_string());
    }

    let scrcpy_bin = get_scrcpy_dir().join("bin");
    if scrcpy_bin.exists() {
        paths.push(scrcpy_bin.to_string_lossy().to_string());
    }

    let ffmpeg_bin = get_ffmpeg_dir().join("bin");
    if ffmpeg_bin.exists() {
        paths.push(ffmpeg_bin.to_string_lossy().to_string());
    }

    let adb_dir = get_adb_dir();
    if adb_dir.exists() {
        paths.push(adb_dir.to_string_lossy().to_string());
    }

    if let Ok(system_path) = std::env::var("PATH") {
        paths.push(system_path);
    }

    paths.join(if cfg!(windows) { ";" } else { ":" })
}

/// Install ADB (Android SDK Platform Tools)
pub async fn install_adb() -> Result<(), String> {
    if is_adb_installed() {
        info!("ADB already installed");
        return Ok(());
    }

    // Try system install first
    if try_system_install("adb").await.is_ok() {
        info!("ADB installed via system package manager");
        return Ok(());
    }

    info!("Installing ADB (Android SDK Platform Tools)...");

    let os = OS::current();
    let url = get_adb_url(os);
    let dest_dir = get_adb_dir();

    let temp_dir = get_companion_dir().join("temp");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let temp_file = temp_dir.join("platform-tools.zip");

    download_file(url, &temp_file).await?;
    extract_archive(&temp_file, &dest_dir, true)?;

    let _ = std::fs::remove_file(&temp_file);

    make_executable(&dest_dir)?;

    info!("ADB installed successfully at {:?}", get_adb_bin());
    Ok(())
}

/// Install all dependencies. Returns a multi-line summary string for frontend logging.
pub async fn install_all() -> Result<String, String> {
    let mut lines = Vec::new();
    lines.push("Installing mirror dependencies...".to_string());

    // Node.js
    match install_nodejs().await {
        Ok(_) => lines.push("✅ Node.js installed".to_string()),
        Err(e) => lines.push(format!("❌ Node.js failed: {}", e)),
    }

    // scrcpy
    match install_scrcpy().await {
        Ok(_) => lines.push("✅ scrcpy installed".to_string()),
        Err(e) => lines.push(format!("❌ scrcpy failed: {}", e)),
    }

    // ffmpeg
    match install_ffmpeg().await {
        Ok(_) => lines.push("✅ ffmpeg installed".to_string()),
        Err(e) => lines.push(format!("❌ ffmpeg failed: {}", e)),
    }

    // ws-scrcpy
    match install_ws_scrcpy().await {
        Ok(_) => lines.push("✅ ws-scrcpy installed".to_string()),
        Err(e) => lines.push(format!("❌ ws-scrcpy failed: {}", e)),
    }

    // ADB
    match install_adb().await {
        Ok(_) => lines.push("✅ ADB installed".to_string()),
        Err(e) => lines.push(format!("❌ ADB failed: {}", e)),
    }

    let summary = lines.join("\n");
    info!("Install summary:\n{}", summary);
    Ok(summary)
}

// ─── Status ──────────────────────────────────────────────────────────────────

/// Get status of all dependencies
#[derive(Debug, Clone, serde::Serialize)]
pub struct DependencyStatus {
    pub nodejs: bool,
    pub scrcpy: bool,
    pub ffmpeg: bool,
    pub ws_scrcpy: bool,
    pub adb: bool,
    pub all_installed: bool,
    pub companion_dir: String,
}

impl DependencyStatus {
    pub fn check() -> Self {
        let status = Self {
            nodejs: is_nodejs_installed(),
            scrcpy: is_scrcpy_installed(),
            ffmpeg: is_ffmpeg_installed(),
            ws_scrcpy: is_ws_scrcpy_installed(),
            adb: is_adb_installed(),
            all_installed: are_all_deps_installed(),
            companion_dir: get_companion_dir().to_string_lossy().to_string(),
        };

        info!("Dependency status: Node.js={}, scrcpy={}, ffmpeg={}, ws-scrcpy={}, ADB={}",
              status.nodejs, status.scrcpy, status.ffmpeg, status.ws_scrcpy, status.adb);

        status
    }
}
