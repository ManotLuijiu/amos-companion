use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

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

/// Result returned to frontend after starting scrcpy
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScrcpyLaunchResult {
    pub success: bool,
    pub pid: Option<u32>,
    pub window_title: String,
    pub message: String,
    pub focus_attempted: bool,
    pub focus_succeeded: Option<bool>,
}

/// Scrcpy stream state
pub struct ScrcpyStream {
    pub process: Option<Child>,
    pub serial: String,
    pub pid: Option<u32>,
    pub active: bool,
    pub stderr_output: Arc<Mutex<String>>,
}

impl ScrcpyStream {
    pub fn new(serial: String) -> Self {
        Self {
            process: None,
            serial,
            pid: None,
            active: false,
            stderr_output: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Start scrcpy stream with proper stderr capture and process verification
    pub async fn start(&mut self) -> ScrcpyLaunchResult {
        if self.active {
            return ScrcpyLaunchResult {
                success: false,
                pid: None,
                window_title: format!("AMOS - {}", self.serial),
                message: "scrcpy is already running".to_string(),
                focus_attempted: false,
                focus_succeeded: None,
            };
        }

        let scrcpy_path = match find_scrcpy() {
            Some(p) => p,
            None => {
                return ScrcpyLaunchResult {
                    success: false,
                    pid: None,
                    window_title: format!("AMOS - {}", self.serial),
                    message: "scrcpy not found. Install with: brew install scrcpy".to_string(),
                    focus_attempted: false,
                    focus_succeeded: None,
                };
            }
        };

        info!("Starting scrcpy stream for device: {}", self.serial);

        let window_title = format!("AMOS - {}", self.serial);

        let mut cmd = Command::new(&scrcpy_path);
        cmd.args([
            "-s",
            &self.serial,
            "--window-title",
            window_title.as_str(),
            "--always-on-top",
            "--stay-awake",
            "--turn-screen-off",
            "--show-touches",
            "--prefer-text",
        ]);

        // Capture stderr so we can detect failures
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());

        // Use piped stderr to detect errors
        let child = match cmd.stderr(Stdio::piped()).spawn() {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to spawn scrcpy: {}", e);
                return ScrcpyLaunchResult {
                    success: false,
                    pid: None,
                    window_title: window_title.clone(),
                    message: format!("Failed to spawn scrcpy: {}", e),
                    focus_attempted: false,
                    focus_succeeded: None,
                };
            }
        };

        let pid = child.id();
        self.pid = Some(pid);
        info!("scrcpy spawned with PID: {:?}", pid);

        // Capture stderr in background using std::io since ChildStderr is sync
        let stderr_arc = self.stderr_output.clone();
        let mut child = child;
        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    warn!("scrcpy stderr: {}", line);
                    let mut buf = stderr_arc.blocking_lock();
                    buf.push_str(&line);
                    buf.push('\n');
                }
            });
        }

        // Give scrcpy a moment to fail or stabilize
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Check if process is still alive
        let status_check = child.try_wait();

        match status_check {
            Ok(Some(status)) => {
                error!("scrcpy exited immediately with status: {}", status);
                let stderr_content = self.stderr_output.lock().await.clone();
                return ScrcpyLaunchResult {
                    success: false,
                    pid: Some(pid),
                    window_title: window_title.clone(),
                    message: format!(
                        "scrcpy exited immediately. stderr: {}",
                        if stderr_content.is_empty() {
                            "(empty)".to_string()
                        } else {
                            stderr_content
                        }
                    ),
                    focus_attempted: false,
                    focus_succeeded: None,
                };
            }
            Ok(None) => {
                // Still alive, continue
            }
            Err(e) => {
                error!("Failed to check scrcpy status: {}", e);
                return ScrcpyLaunchResult {
                    success: false,
                    pid: Some(pid),
                    window_title: window_title.clone(),
                    message: format!("Failed to verify scrcpy is running: {}", e),
                    focus_attempted: false,
                    focus_succeeded: None,
                };
            }
        }

        // Process is alive - store it
        self.process = Some(child);
        self.active = true;

        // Try to focus the scrcpy window on macOS
        let focus_result = self.try_focus_window(&window_title);

        ScrcpyLaunchResult {
            success: true,
            pid: Some(pid),
            window_title: window_title.clone(),
            message: format!(
                "scrcpy started successfully (PID: {}). Window title: '{}'",
                pid, window_title
            ),
            focus_attempted: focus_result.is_some(),
            focus_succeeded: focus_result,
        }
    }

    /// Try to bring the scrcpy window to front on macOS using AppleScript.
    /// Returns Some(true/false) if focus was attempted (macOS only).
    fn try_focus_window(&self, window_title: &str) -> Option<bool> {
        #[cfg(target_os = "macos")]
        {
            let title = window_title.to_string();
            // Try synchronously to get a result back
            let result = std::thread::spawn(move || {
                // Wait for scrcpy to create its window
                std::thread::sleep(Duration::from_millis(800));

                // Method 1: Try to activate scrcpy app by name (most reliable)
                let script1 = r#"tell application "scrcpy" to activate"#;
                let r1 = Command::new("osascript")
                    .arg("-e")
                    .arg(script1)
                    .output();

                // Check if scrcpy was actually activated (no error)
                let activated = match r1 {
                    Ok(out) => out.status.success(),
                    Err(_) => false,
                };

                if activated {
                    info!("scrcpy activated via AppleScript");
                } else {
                    warn!("AppleScript activation failed - Accessibility permission may be needed");
                }

                activated
            })
            .join();

            match result {
                Ok(success) => Some(success),
                Err(_) => {
                    warn!("AppleScript thread panicked");
                    Some(false)
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = window_title; // silence unused warning
            let _ = self;
            None // Not on macOS, no focus attempt
        }
    }

    /// Stop the scrcpy stream
    pub fn stop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.pid = None;
        self.active = false;
        info!("scrcpy stopped for {}", self.serial);
    }

    /// Check if stream is running
    pub fn is_active(&mut self) -> bool {
        if let Some(child) = self.process.as_mut() {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    self.active = false;
                    false
                }
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }
}

/// Shared scrcpy manager
pub type ScrcpyManager = Arc<Mutex<ScrcpyStream>>;

pub fn create_scrcpy_manager(serial: String) -> ScrcpyManager {
    Arc::new(Mutex::new(ScrcpyStream::new(serial)))
}
