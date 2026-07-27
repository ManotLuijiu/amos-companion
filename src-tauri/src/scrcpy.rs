use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{error, info};

/// Find scrcpy binary in common locations
pub fn find_scrcpy() -> Option<PathBuf> {
    let candidates = [
        "/usr/local/bin/scrcpy",
        "/opt/homebrew/bin/scrcpy",
        "/usr/bin/scrcpy",
    ];

    for path in &candidates {
        if PathBuf::from(path).exists() {
            info!("Found scrcpy at: {}", path);
            return Some(PathBuf::from(path));
        }
    }

    // Try which command
    if let Ok(output) = Command::new("which").arg("scrcpy").output() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            let path_buf = PathBuf::from(&path);
            if path_buf.exists() {
                info!("Found scrcpy via which: {}", path);
                return Some(path_buf);
            }
        }
    }

    None
}

/// Check if scrcpy is installed
pub fn is_scrcpy_available() -> bool {
    find_scrcpy().is_some()
}

/// Get scrcpy version
pub fn get_scrcpy_version() -> Option<String> {
    info!("Checking scrcpy version");
    let scrcpy_path = find_scrcpy()?;
    let output = Command::new(&scrcpy_path).arg("--version").output().ok()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        info!("scrcpy version: {}", version);
        Some(version)
    } else {
        None
    }
}

/// Scrcpy stream state
pub struct ScrcpyStream {
    pub process: Option<Child>,
    pub serial: String,
    pub active: bool,
}

impl ScrcpyStream {
    pub fn new(serial: String) -> Self {
        Self {
            process: None,
            serial,
            active: false,
        }
    }

    /// Start scrcpy stream to a local file descriptor for WebSocket
    pub fn start(&mut self) -> Result<(), String> {
        if self.active {
            return Err("Stream already active".to_string());
        }

        let scrcpy_path = find_scrcpy()
            .ok_or_else(|| "scrcpy not found. Install with: brew install scrcpy".to_string())?;

        info!("Starting scrcpy stream for device: {}", self.serial);

        let window_title = format!("AMOS - {}", self.serial);

        // scrcpy with video output to stdout, we'll pipe it
        // Use tcpip for forwarding if needed, direct USB otherwise
        let mut cmd = Command::new(&scrcpy_path);
        cmd.args([
            "-s",
            &self.serial,
            "--window-title",
            &window_title,
            "--always-on-top",
            "--stay-awake",
            "--turn-screen-off",
            "--show-touches",
            "--prefer-text",
            "--quiet",
        ]);

        // For WS mode, we'd need scrcpy-server approach
        // For now, just launch scrcpy window
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        match cmd.spawn() {
            Ok(child) => {
                self.process = Some(child);
                self.active = true;
                info!("scrcpy started successfully for {}", self.serial);

                // Try to focus the scrcpy window on macOS
                self.try_focus_window(&window_title);

                Ok(())
            }
            Err(e) => {
                error!("Failed to start scrcpy: {}", e);
                Err(format!("Failed to start scrcpy: {}", e))
            }
        }
    }

    /// Try to bring the scrcpy window to front on macOS using AppleScript
    fn try_focus_window(&self, window_title: &str) {
        #[cfg(target_os = "macos")]
        {
            // Give scrcpy a moment to create the window
            let title = window_title.to_string();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(500));
                // Use AppleScript to bring the scrcpy window to front
                // Try to focus scrcpy process window by its PID
                let script = format!(
                    r#"
tell application "System Events"
    set frontmost of every process whose unix id is (do shell script "pgrep -x scrcpy | head -1") to true
end tell
"#
                );
                let _ = Command::new("osascript").arg("-e").arg(&script).output();
            });
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = window_title; // silence unused warning
            let _ = self;
        }
    }

    /// Stop the scrcpy stream
    pub fn stop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.active = false;
        info!("scrcpy stopped for {}", self.serial);
    }

    /// Check if stream is running
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// Shared scrcpy manager
pub type ScrcpyManager = Arc<Mutex<ScrcpyStream>>;

pub fn create_scrcpy_manager(serial: String) -> ScrcpyManager {
    Arc::new(Mutex::new(ScrcpyStream::new(serial)))
}
