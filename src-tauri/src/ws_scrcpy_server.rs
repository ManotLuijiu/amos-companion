//! ws-scrcpy Server Manager
//!
//! Manages the ws-scrcpy Node.js server that provides browser-based mirroring.

use std::process::{Child, Command};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::dependency_manager::{
    get_companion_dir, get_npm_bin, get_path_env, get_ws_scrcpy_dir, is_ws_scrcpy_installed,
};

/// Default port for ws-scrcpy server
pub const DEFAULT_PORT: u16 = 8000;

/// Default host to bind to
pub const DEFAULT_HOST: &str = "0.0.0.0";

/// ws-scrcpy server state
pub struct WsScrcpyServer {
    process: Option<Child>,
    port: u16,
    running: bool,
}

impl Default for WsScrcpyServer {
    fn default() -> Self {
        Self::new()
    }
}

impl WsScrcpyServer {
    pub fn new() -> Self {
        Self {
            process: None,
            port: DEFAULT_PORT,
            running: false,
        }
    }

    /// Check if ws-scrcpy server is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get the server URL
    pub fn get_url(&self) -> String {
        format!("http://{}:{}", DEFAULT_HOST, self.port)
    }

    /// Start the ws-scrcpy server
    pub async fn start(&mut self) -> Result<String, String> {
        if self.running {
            return Ok(self.get_url());
        }

        // Check if installed
        if !is_ws_scrcpy_installed() {
            return Err("ws-scrcpy not installed. Run install_deps() first.".to_string());
        }

        let ws_dir = get_ws_scrcpy_dir();
        let npm = get_npm_bin();

        if !npm.exists() {
            return Err(format!(
                "npm not found at {:?}. Is Node.js installed?",
                npm
            ));
        }

        info!("Starting ws-scrcpy server on port {}...", self.port);

        // Start npm with server.js
        let mut cmd = Command::new(&npm);
        cmd.arg("start")
            .current_dir(&ws_dir)
            .env("PATH", get_path_env())
            .env("HOST", DEFAULT_HOST)
            .env("PORT", self.port.to_string());

        // Redirect output to log files
        let log_dir = get_companion_dir().join("logs");
        std::fs::create_dir_all(&log_dir).ok();

        let stdout_file = log_dir.join("ws-scrcpy-stdout.log");
        let stderr_file = log_dir.join("ws-scrcpy-stderr.log");

        let stdout = std::fs::File::create(&stdout_file)
            .map_err(|e| format!("Failed to create stdout log: {}", e))?;
        let stderr = std::fs::File::create(&stderr_file)
            .map_err(|e| format!("Failed to create stderr log: {}", e))?;

        cmd.stdout(std::process::Stdio::from(stdout));
        cmd.stderr(std::process::Stdio::from(stderr));

        // Start the process
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to start ws-scrcpy: {}", e))?;

        // Wait a moment for server to start
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Check if still running
        match child.try_wait() {
            Ok(Some(status)) => {
                error!("ws-scrcpy exited immediately with status: {}", status);
                return Err(format!(
                    "ws-scrcpy exited with status: {}. Check logs at {:?}",
                    status, log_dir
                ));
            }
            Ok(None) => {
                // Still running, good!
            }
            Err(e) => {
                error!("Failed to check ws-scrcpy status: {}", e);
                return Err(format!("Failed to verify ws-scrcpy is running: {}", e));
            }
        }

        self.process = Some(child);
        self.running = true;

        let url = self.get_url();
        info!("ws-scrcpy server started at {}", url);

        Ok(url)
    }

    /// Stop the ws-scrcpy server
    pub fn stop(&mut self) {
        if let Some(mut child) = self.process.take() {
            info!("Stopping ws-scrcpy server...");
            let _ = child.kill();
            let _ = child.wait();
        }
        self.running = false;
        info!("ws-scrcpy server stopped");
    }

    /// Restart the server
    pub async fn restart(&mut self) -> Result<String, String> {
        self.stop();
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        self.start().await
    }

    /// Check if server is still running
    pub fn check_alive(&mut self) -> bool {
        if let Some(ref mut child) = self.process {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    self.running = false;
                    false
                }
                Ok(None) => {
                    // Still running
                    true
                }
                Err(_) => {
                    // Can't check, assume dead
                    self.running = false;
                    false
                }
            }
        } else {
            false
        }
    }

    /// Set custom port
    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }
}

impl Drop for WsScrcpyServer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Shared ws-scrcpy server manager
pub type WsScrcpyServerManager = Arc<Mutex<WsScrcpyServer>>;

/// Create a new ws-scrcpy server manager
pub fn create_ws_scrcpy_server() -> WsScrcpyServerManager {
    Arc::new(Mutex::new(WsScrcpyServer::new()))
}

/// Status information for the frontend
#[derive(Debug, Clone, serde::Serialize)]
pub struct WsScrcpyStatus {
    pub running: bool,
    pub url: String,
    pub port: u16,
    pub installed: bool,
}

impl From<&WsScrcpyServer> for WsScrcpyStatus {
    fn from(server: &WsScrcpyServer) -> Self {
        Self {
            running: server.running,
            url: server.get_url(),
            port: server.port,
            installed: is_ws_scrcpy_installed(),
        }
    }
}

impl WsScrcpyServer {
    pub fn get_status(&self) -> WsScrcpyStatus {
        WsScrcpyStatus::from(self)
    }
}
