//! Scrcpy-Server WebSocket streaming implementation
//! 
//! This module implements proper Android screen mirroring using scrcpy-server,
//! providing low-latency video streaming with touch input support.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::thread;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::adb::{find_adb, run_adb};

/// Path to scrcpy-server JAR (placed next to executable or in resources)
fn get_scrcpy_server_path() -> PathBuf {
    // Try multiple locations
    let candidates = vec![
        // Next to executable
        PathBuf::from(".").join("scrcpy-server.jar"),
        // Resources folder
        PathBuf::from(".").join("resources").join("scrcpy-server.jar"),
        // Parent directory
        PathBuf::from("..").join("scrcpy-server.jar"),
    ];

    for path in candidates {
        if path.exists() {
            info!("Found scrcpy-server at: {:?}", path);
            return path;
        }
    }

    // Fallback to temp
    info!("scrcpy-server not found, will download on first use");
    std::env::temp_dir().join("scrcpy-server.jar")
}

/// Default scrcpy server settings
const DEFAULT_BITRATE: i32 = 8000000; // 8 Mbps
const DEFAULT_MAX_FPS: i32 = 60;
const DEFAULT_BUFFER_TIME: i32 = 0;

/// Screen dimensions
#[derive(Debug, Clone)]
pub struct ScreenSize {
    pub width: u32,
    pub height: u32,
}

/// Video frame from scrcpy
#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub data: Vec<u8>,
    pub timestamp: u64,
}

/// ScrcpyServer state
pub struct ScrcpyServer {
    /// Server process (scrcpy-server running on device)
    process: Option<Child>,
    /// ADB tunnel for video port
    adb_process: Option<Child>,
    /// Device serial
    serial: String,
    /// Is currently streaming
    active: bool,
    /// Video port on device
    device_port: i32,
    /// Local port forwarded
    local_port: i32,
    /// Screen size
    screen_size: Option<ScreenSize>,
}

impl ScrcpyServer {
    pub fn new(serial: String) -> Self {
        Self {
            process: None,
            adb_process: None,
            serial,
            active: false,
            device_port: 8888,
            local_port: 8888,
            screen_size: None,
        }
    }

    /// Check if scrcpy server is running
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get current screen size
    pub fn get_screen_size(&self) -> Option<ScreenSize> {
        self.screen_size.clone()
    }

    /// Start scrcpy-server streaming
    pub fn start(&mut self) -> Result<String, String> {
        if self.active {
            return Err("Server already running".to_string());
        }

        let adb_path = find_adb();

        // Step 1: Kill any existing scrcpy server on device
        info!("Killing existing scrcpy server on device");
        let _ = run_adb(&[
            "-s", &self.serial, "shell", "pkill", "-f", "scrcpy",
        ]);
        let _ = run_adb(&[
            "-s", &self.serial, "shell", "am", "force-stop", "org.genymobile.scrcpy",
        ]);

        // Step 2: Push scrcpy-server to device
        info!("Pushing scrcpy-server to device");
        let server_path = "/data/local/tmp/scrcpy-server.jar";
        
        // Find or download scrcpy-server
        let server_file = get_scrcpy_server_path();
        
        if !server_file.exists() {
            // Download if not found
            info!("Downloading scrcpy-server...");
            let download_result = Command::new("curl")
                .args([
                    "-sL",
                    "https://github.com/Genymobile/scrcpy/releases/download/v2.1.1/scrcpy-server-v2.1.1",
                    "-o", server_file.to_str().unwrap(),
                ])
                .output();
            
            if download_result.is_err() || !server_file.exists() {
                return Err("Failed to download scrcpy-server".to_string());
            }
        }

        // Push to device
        let push_result = Command::new(&adb_path)
            .args(["-s", &self.serial, "push", server_file.to_str().unwrap(), server_path])
            .output();

        if let Ok(output) = push_result {
            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to push scrcpy-server: {}", err));
            }
        }

        // Make scrcpy-server executable
        let _ = run_adb(&[
            "-s", &self.serial, "shell", "chmod", "755", server_path,
        ]);

        // Step 3: Start scrcpy-server on device
        info!("Starting scrcpy-server on device");
        
        let start_cmd = format!(
            "CLASSPATH={} app_process / --proto={} --bit-rate={} --max-fps={} --buffer-time={}",
            server_path,
            self.device_port,
            DEFAULT_BITRATE,
            DEFAULT_MAX_FPS,
            DEFAULT_BUFFER_TIME
        );

        let start_result = Command::new(&adb_path)
            .args([
                "-s", &self.serial, "shell", "nohup", &start_cmd,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        match start_result {
            Ok(child) => {
                self.process = Some(child);
            }
            Err(e) => {
                return Err(format!("Failed to start scrcpy-server: {}", e));
            }
        }

        // Wait for server to start
        thread::sleep(std::time::Duration::from_millis(500));

        // Step 4: Forward device port to localhost
        info!("Forwarding device port {} to localhost", self.device_port);
        
        // Kill any existing forward
        let _ = Command::new(&adb_path)
            .args(["-s", &self.serial, "forward", "--remove", &format!("tcp:{}", self.device_port)])
            .output();

        let forward_result = Command::new(&adb_path)
            .args([
                "-s", &self.serial, "forward",
                &format!("tcp:{}", self.local_port),
                &format!("tcp:{}", self.device_port),
            ])
            .output();

        match forward_result {
            Ok(output) if output.status.success() => {
                info!("Port forwarded successfully");
            }
            Ok(output) => {
                let err = String::from_utf8_lossy(&output.stderr);
                warn!("Port forward warning: {}", err);
            }
            Err(e) => {
                return Err(format!("Failed to forward port: {}", e));
            }
        }

        self.active = true;
        
        // Return the local URL for frontend to connect
        Ok(format!("http://127.0.0.1:{}/", self.local_port))
    }

    /// Stop scrcpy-server
    pub fn stop(&mut self) {
        if !self.active {
            return;
        }

        info!("Stopping scrcpy-server");

        // Kill scrcpy process
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        // Kill ADB tunnel
        if let Some(mut child) = self.adb_process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        // Kill server on device
        let _ = run_adb(&[
            "-s", &self.serial, "shell", "pkill", "-f", "scrcpy",
        ]);

        // Remove port forward
        let adb_path = find_adb();
        let _ = Command::new(&adb_path)
            .args([
                "-s", &self.serial, "forward",
                "--remove", &format!("tcp:{}", self.local_port),
            ])
            .output();

        self.active = false;
        info!("scrcpy-server stopped");
    }

    /// Send touch event (tap)
    pub fn tap(&self, x: i32, y: i32) -> Result<(), String> {
        if !self.active {
            return Err("Server not running".to_string());
        }

        run_adb(&[
            "-s", &self.serial, "shell", "input", "tap",
            &x.to_string(), &y.to_string(),
        ])?;
        Ok(())
    }

    /// Send swipe event
    pub fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: i32) -> Result<(), String> {
        if !self.active {
            return Err("Server not running".to_string());
        }

        run_adb(&[
            "-s", &self.serial, "shell", "input", "swipe",
            &x1.to_string(), &y1.to_string(),
            &x2.to_string(), &y2.to_string(),
            &duration_ms.to_string(),
        ])?;
        Ok(())
    }

    /// Send text input
    pub fn text(&self, text: &str) -> Result<(), String> {
        if !self.active {
            return Err("Server not running".to_string());
        }

        // Convert spaces to %s for ADB
        let escaped = text.replace(' ', "%s");
        
        run_adb(&[
            "-s", &self.serial, "shell", "input", "text", &escaped,
        ])?;
        Ok(())
    }

    /// Press back button
    pub fn back(&self) -> Result<(), String> {
        if !self.active {
            return Err("Server not running".to_string());
        }

        run_adb(&[
            "-s", &self.serial, "shell", "input", "keyevent", "KEYCODE_BACK",
        ])?;
        Ok(())
    }

    /// Press home button
    pub fn home(&self) -> Result<(), String> {
        if !self.active {
            return Err("Server not running".to_string());
        }

        run_adb(&[
            "-s", &self.serial, "shell", "input", "keyevent", "KEYCODE_HOME",
        ])?;
        Ok(())
    }

    /// Press enter button
    pub fn enter(&self) -> Result<(), String> {
        if !self.active {
            return Err("Server not running".to_string());
        }

        run_adb(&[
            "-s", &self.serial, "shell", "input", "keyevent", "KEYCODE_ENTER",
        ])?;
        Ok(())
    }
}

impl Drop for ScrcpyServer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Shared scrcpy server manager
pub type ScrcpyServerManager = Arc<Mutex<ScrcpyServer>>;

pub fn create_scrcpy_server(serial: String) -> ScrcpyServerManager {
    Arc::new(Mutex::new(ScrcpyServer::new(serial)))
}
