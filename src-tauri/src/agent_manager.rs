use crate::adb::find_adb;
use crate::device_agent_installer;
use crate::AgentStatus;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tracing::{error, info, warn};

pub struct AgentManager {
    /// Currently running device-agent process, if any.
    process: Option<Child>,
    /// OS process ID of the running agent.
    pid: Option<u32>,
    /// Last error message from the agent.
    error_message: Option<String>,
    /// Last detected ADB devices.
    connected_devices: Vec<String>,
}

impl AgentManager {
    pub fn new() -> Self {
        Self {
            process: None,
            pid: None,
            error_message: None,
            connected_devices: vec![],
        }
    }

    /// Returns true if the agent process is currently running.
    /// Process is running if try_wait() returns Ok(None) (no exit status yet).
    /// Process has exited if try_wait() returns Ok(Some(exit_status)).
    pub fn is_running(&mut self) -> bool {
        self.process
            .as_mut()
            .map(|p| p.try_wait().ok().flatten().is_none())
            .unwrap_or(false)
    }

    /// Returns true if the agent process has exited (has an exit status).
    /// This is the inverse of is_running() - used to check for immediate exit.
    fn has_exited(&mut self) -> bool {
        !self.is_running()
    }

    /// Start the `uv run python -m amos_device_agent` process.
    pub async fn start(
        &mut self,
        api_url: &str,
        agent_id: &str,
        device_key: Option<String>,
        device_secret: Option<String>,
        workspace_id: Option<String>,
    ) -> Result<(), AgentError> {
        info!("start() called, checking if already running...");
        if self.is_running() {
            warn!("Agent already running!");
            return Err(AgentError::AlreadyRunning);
        }

        self.error_message = None;

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

        info!(
            "Device-agent working dir: {:?}",
            agent_cwd
        );

        info!("Checking if uv is available...");
        let uv_check = Command::new("sh")
            .arg("-c")
            .arg("which uv")
            .output()
            .await;
        match &uv_check {
            Ok(out) => info!("uv path check: {}", String::from_utf8_lossy(&out.stdout)),
            Err(e) => warn!("uv check failed: {}", e),
        }

        // Clone for both uses since command takes ownership
        let agent_cwd_clone = agent_cwd.clone();

        // Try uv first; fall back to python3 if uv is not available
        let mut cmd = Command::new("uv");
        cmd.arg("run")
            .arg("python")
            .arg("-m")
            .arg("amos_device_agent")
            .env("AMOS_API_URL", api_url)
            .env("AMOS_AGENT_ID", agent_id)
            .current_dir(agent_cwd_clone)
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

        info!("Attempting to spawn device-agent via uv...");
        let child = match cmd.spawn() {
            Ok(c) => {
                info!("Successfully spawned via uv!");
                c
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                warn!("uv not found, falling back to python3");
                let mut fallback = Command::new("python3");
                fallback
                    .arg("-m")
                    .arg("amos_device_agent")
                    .env("AMOS_API_URL", api_url)
                    .env("AMOS_AGENT_ID", agent_id)
                    .current_dir(agent_cwd)
                    .stdout(Stdio::from(
                        std::fs::File::create(&stdout_file)
                            .map_err(|e| AgentError::SpawnFailed(format!("stdout: {}", e)))?,
                    ))
                    .stderr(Stdio::from(
                        std::fs::File::create(&stderr_file)
                            .map_err(|e| AgentError::SpawnFailed(format!("stderr: {}", e)))?,
                    ));
                // Add optional device-agent auth credentials
                if let Some(key) = &device_key {
                    fallback.env("AMOS_API_KEY", key);
                }
                if let Some(secret) = &device_secret {
                    fallback.env("AMOS_API_SECRET", secret);
                }
                if let Some(ref ws_id) = workspace_id {
                    fallback.env("AMOS_WORKSPACE_ID", ws_id);
                }
                fallback.spawn().map_err(|e| AgentError::SpawnFailed(e.to_string()))?
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

        // Brief wait then check if still running
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        if self.has_exited() {
            // Process exited immediately - check logs
            let log_content = std::fs::read_to_string(&stderr_file)
                .unwrap_or_else(|_| "Could not read log".to_string());
            let stdout_content = std::fs::read_to_string(&stdout_file)
                .unwrap_or_else(|_| "Could not read stdout log".to_string());
            error!("Agent process exited immediately! stderr: {}, stdout: {}", log_content, stdout_content);
            self.error_message = Some(format!("Agent exited immediately: check logs at {:?}", stderr_file));
            // Clear the process since it exited
            self.process = None;
            self.pid = None;
            // Return an error so frontend knows startup failed
            return Err(AgentError::StartupFailed(log_content));
        } else {
            info!("Agent process is running");
        }

        Ok(())
    }

    /// Stop the running device-agent process.
    pub async fn stop(&mut self) {
        if let Some(mut child) = self.process.take() {
            info!("Sending SIGTERM to device-agent PID {:?}", self.pid);
            let _ = child.kill().await;
            let _ = child.wait().await;
            self.pid = None;
            info!("Device-agent stopped");
        }
    }

    /// Return the current agent status.
    pub fn get_status(&mut self) -> AgentStatus {
        // Query connected ADB devices
        let connected_devices = Self::get_connected_devices();

        AgentStatus {
            agent_online: true,
            agent_running: self.is_running(),
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
