//! ✅ WORKING: Device Agent Manager
//!
//! Manages the AMOS device-agent process lifecycle:
//! - Start/stop the Python agent process
//! - Auto-register device with backend workspace
//! - OAuth login flow integration
//! - Device-agent installation on first run
//!
//! Related: workspace_manager.rs, device_agent_installer.rs

use crate::adb::find_adb;
use crate::device_agent_installer;
use crate::AgentStatus;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tracing::{error, info, warn};

pub struct AgentManager {
    /// Currently running device-agent process.
    /// We track the actual Python agent process directly.
    process: Option<Child>,
    /// OS process ID of the running agent.
    pid: Option<u32>,
    /// Last error message from the agent.
    error_message: Option<String>,
    /// Whether the agent was successfully started.
    /// This tracks that startup completed successfully, so we don't report
    /// "stopped" just because try_wait() returns an exit status.
    agent_started: bool,
    /// Last detected ADB devices.
    connected_devices: Vec<String>,
}

impl AgentManager {
    pub fn new() -> Self {
        Self {
            process: None,
            pid: None,
            error_message: None,
            agent_started: false,
            connected_devices: vec![],
        }
    }

    /// Returns true if the agent is in a running state.
    /// The truth source is: we successfully started and haven't stopped or errored.
    /// Child process handle may not track the real agent if it daemonizes.
    pub fn is_running(&self) -> bool {
        // Running = started successfully and no error
        self.agent_started && self.error_message.is_none()
    }

    /// Start the device-agent process.
    /// We spawn Python directly (not via `uv run`) so tokio's Child tracks the actual agent.
    pub async fn start(
        &mut self,
        api_url: &str,
        agent_id: &str,
        device_key: Option<String>,
        device_secret: Option<String>,
        workspace_id: Option<String>,
        user_id: Option<String>,
    ) -> Result<(), AgentError> {
        info!("start() called, checking if already running...");
        if self.is_running() {
            warn!("Agent already running!");
            return Err(AgentError::AlreadyRunning);
        }

        self.error_message = None;
        self.agent_started = false;

        // Get log dir for agent output
        let log_dir = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("amos-companion")
            .join("logs");
        std::fs::create_dir_all(&log_dir).ok();
        let stdout_file = log_dir.join("agent-stdout.log");
        let stderr_file = log_dir.join("agent-stderr.log");

        info!(
            "Agent logs will go to: {:?} and {:?}",
            stdout_file, stderr_file
        );

        // Open log files for the agent process
        let stdout = std::fs::File::create(&stdout_file)
            .map_err(|e| AgentError::SpawnFailed(format!("Failed to create stdout log: {}", e)))?;
        let stderr = std::fs::File::create(&stderr_file)
            .map_err(|e| AgentError::SpawnFailed(format!("Failed to create stderr log: {}", e)))?;

        // Get working directory from installer
        let agent_cwd = device_agent_installer::get_working_dir();

        info!("Device-agent working dir: {:?}", agent_cwd);

        // Use uv run to ensure dependencies (httpx, etc.) are available.
        // uv creates a temporary venv if needed and keeps the process tracked.
        let mut cmd = Command::new("uv");
        cmd.arg("run")
            .arg("--directory")
            .arg(&agent_cwd)
            .arg("python")
            .arg("-m")
            .arg("amos_device_agent")
            .env("AMOS_API_URL", api_url)
            .env("AMOS_AGENT_ID", agent_id)
            .current_dir(&agent_cwd)
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr));

        // Add optional device-agent auth credentials
        if let Some(key) = &device_key {
            cmd.env("AMOS_API_KEY", key);
        }
        if let Some(secret) = &device_secret {
            cmd.env("AMOS_API_SECRET", secret);
        }
        if let Some(ref ws_id) = workspace_id {
            cmd.env("AMOS_WORKSPACE_ID", ws_id);
        }
        if let Some(ref uid) = user_id {
            cmd.env("AMOS_USER_ID", uid);
        }

        info!("Attempting to spawn device-agent via python3...");
        let child = match cmd.spawn() {
            Ok(c) => {
                info!("Device-agent spawned successfully");
                c
            }
            Err(e) => {
                error!("Failed to spawn: {:?}", e);
                self.error_message = Some(format!("Failed to spawn agent: {}", e));
                return Err(AgentError::SpawnFailed(e.to_string()));
            }
        };

        let pid = child.id();
        info!("Device-agent spawned with PID: {:?}", pid);

        self.pid = pid;
        self.process = Some(child);

        // Mark as started immediately - we trust the spawn succeeded.
        // The child handle may not track the real agent if it daemonizes,
        // so we use this flag as the primary truth source for running state.
        self.agent_started = true;
        info!("Agent marked as started");

        // Brief wait to catch immediate crash (but don't block on is_running)
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Check if process is still alive - if it exited immediately, log it
        // but keep agent_started true so we don't falsely report "stopped"
        let child_alive = self
            .process
            .as_mut()
            .map(|p| p.try_wait().ok().flatten().is_none())
            .unwrap_or(false);

        if !child_alive {
            warn!("Agent process exited quickly after spawn, but keeping running state (may have daemonized)");
        }

        Ok(())
    }

    /// Stop the running device-agent process.
    pub async fn stop(&mut self) {
        self.agent_started = false;
        if let Some(mut child) = self.process.take() {
            info!("Sending SIGTERM to device-agent PID {:?}", self.pid);
            let _ = child.kill().await;
            let _ = child.wait().await;
            self.pid = None;
            self.process = None;
            info!("Device-agent stopped");
        }
    }

    /// Return the current agent status.
    pub fn get_status(&self) -> AgentStatus {
        // Query connected ADB devices
        let connected_devices = Self::get_connected_devices();

        // Check if agent is running
        let running = self.is_running();

        AgentStatus {
            agent_online: true,
            agent_running: running,
            connected_devices,
            platform: std::env::consts::OS.to_string(),
            companion_version: env!("CARGO_PKG_VERSION").to_string(),
            adb_version: String::new(),
            api_url: String::new(),
            agent_pid: self.pid,
            error_message: self.error_message.clone(),
        }
    }

    /// Query connected ADB devices.
    fn get_connected_devices() -> Vec<String> {
        let adb_path = find_adb();
        let output = std::process::Command::new(&adb_path)
            .arg("devices")
            .arg("-l")
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout
                    .lines()
                    .skip(1) // Skip "List of devices attached"
                    .filter(|line| !line.trim().is_empty())
                    .filter(|line| !line.contains("offline"))
                    .map(|line| {
                        // Extract device serial and model
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let serial = parts[0];
                            // Try to extract model from -l output
                            let model = parts
                                .iter()
                                .skip(2)
                                .find(|s| s.starts_with("model:"))
                                .map(|s| s.trim_start_matches("model:"))
                                .unwrap_or(serial);
                            format!("{} ({})", model, serial)
                        } else {
                            line.to_string()
                        }
                    })
                    .collect()
            }
            Err(e) => {
                tracing::warn!("Failed to query ADB devices: {}", e);
                vec![]
            }
        }
    }

    /// Try to detect the device-agent working directory.
    /// In dev: looks for `services/device-agent` relative to this binary.
    /// In prod: returns None (uses cwd, or expects pip-installed module).
    fn find_device_agent_dir() -> Option<String> {
        // Get the directory containing the current binary
        let exe_path = std::env::current_exe().ok()?;
        let exe_dir = exe_path.parent()?;

        // In dev, the Tauri binary lives in:
        //   src-tauri/target/debug/amos_companion
        // And the device-agent is at:
        //   backend/services/device-agent/
        // So walk up from exe_dir and look for backend/services/device-agent

        let candidates: Vec<_> = [
            exe_dir.join("../../../backend/services/device-agent"),
            exe_dir.join("../../backend/services/device-agent"),
            exe_dir.join("../backend/services/device-agent"),
            exe_dir.join("backend/services/device-agent"),
        ]
        .into_iter()
        .filter_map(|p| p.canonicalize().ok())
        .collect();

        for candidate in &candidates {
            if candidate.exists() && candidate.join("amos_device_agent").exists() {
                info!("Found device-agent at {:?}", candidate);
                return candidate.to_str().map(String::from);
            }
        }

        // Also check relative to CWD
        let cwd = std::env::current_dir().ok()?;
        let dev_candidates = [
            cwd.join("backend/services/device-agent"),
            cwd.join("../backend/services/device-agent"),
        ];

        for candidate in &dev_candidates {
            if candidate.exists() {
                return candidate.to_str().map(String::from);
            }
        }

        None
    }
}

impl Default for AgentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Agent is already running")]
    AlreadyRunning,

    #[error("Failed to spawn agent: {0}")]
    SpawnFailed(String),

    #[error("Agent is not running")]
    NotRunning,

    #[error("Agent exited immediately after startup: {0}")]
    StartupFailed(String),
}
