//! Video streaming via WebSocket
//! 
//! Implements low-latency screen streaming using:
//! 1. scrcpy-server for h264 encoding on device
//! 2. WebSocket for streaming to frontend
//! 3. Canvas rendering in browser

use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::info;

/// Stream configuration
const DEFAULT_BITRATE: i32 = 8000000;
const DEFAULT_MAX_FPS: i32 = 60;
const DEFAULT_MAX_SIZE: i32 = 1920;

/// Video frame from scrcpy-server
#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub timestamp: u64,
    pub data: Vec<u8>,
}

/// Video stream state (public for use in lib.rs)
pub struct VideoStream {
    pub serial: String,
    pub port: u16,
    running: Arc<Mutex<bool>>,
}

impl VideoStream {
    pub fn new(serial: String) -> Self {
        Self {
            serial,
            port: 8888,
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start video stream
    pub fn start(&mut self) -> Result<u16, String> {
        let adb_path = crate::adb::find_adb();
        
        // 1. Kill existing scrcpy on device
        let _ = Command::new(&adb_path)
            .args(["-s", &self.serial, "shell", "pkill", "-9", "-f", "scrcpy"])
            .output();
        thread::sleep(Duration::from_millis(200));

        // 2. Find available port
        let port = self.find_port();
        self.port = port;

        // 3. Set up ADB reverse (device → localhost)
        let _ = Command::new(&adb_path)
            .args(["-s", &self.serial, "reverse", "--remove", &format!("tcp:{}", port)])
            .output();
            
        let reverse_out = Command::new(&adb_path)
            .args(["-s", &self.serial, "reverse", &format!("tcp:{}", port), &format!("tcp:{}", port)])
            .output();
            
        if let Ok(out) = reverse_out {
            if !out.status.success() {
                return Err(format!("ADB reverse failed: {}", String::from_utf8_lossy(&out.stderr)));
            }
        }

        // 4. Get scrcpy-server path
        let server_path = self.get_server_path()?;
        
        // 5. Push server to device if needed
        let device_path = "/data/local/tmp/scrcpy-server.jar";
        self.push_server(&adb_path, &server_path, device_path)?;

        // 6. Start scrcpy-server
        let cmd = format!(
            "CLASSPATH={} app_process / {} server {} --bit-rate={} --max-fps={} --max-size={}",
            device_path,
            device_path,
            port,
            DEFAULT_BITRATE,
            DEFAULT_MAX_FPS,
            DEFAULT_MAX_SIZE
        );

        let _ = Command::new(&adb_path)
            .args(["-s", &self.serial, "shell", "nohup", &cmd, "> /dev/null 2>&1 &"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        // 7. Wait for server to start
        thread::sleep(Duration::from_secs(1));

        // 8. Mark as running
        *self.running.lock().unwrap() = true;

        info!("Video stream started on port {}", port);
        Ok(port)
    }

    fn find_port(&self) -> u16 {
        for port in [8888, 8889, 8890, 8891, 8892].iter() {
            if TcpStream::connect_timeout(
                &std::net::SocketAddr::from(([127, 0, 0, 1], *port), ),
                Duration::from_millis(50),
            ).is_err() {
                return *port;
            }
        }
        8888
    }

    fn get_server_path(&self) -> Result<String, String> {
        // Check multiple locations
        let candidates = vec![
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.join("scrcpy-server.jar")))
                .map(|p| p.to_string_lossy().to_string()),
            Some("scrcpy-server/scrcpy-server.jar".to_string()),
            Some("src-tauri/scrcpy-server/scrcpy-server.jar".to_string()),
            Some("/tmp/scrcpy-server.jar".to_string()),
        ];

        for path in candidates.into_iter().flatten() {
            if std::path::Path::new(&path).exists() {
                return Ok(path);
            }
        }

        // Try to download
        info!("Downloading scrcpy-server...");
        let out = Command::new("curl")
            .args(["-sL", 
                "https://github.com/Genymobile/scrcpy/releases/download/v2.1.1/scrcpy-server-v2.1.1",
                "-o", "/tmp/scrcpy-server.jar"])
            .output();

        match out {
            Ok(o) if o.status.success() => Ok("/tmp/scrcpy-server.jar".to_string()),
            _ => Err("Failed to download scrcpy-server".to_string()),
        }
    }

    fn push_server(&self, adb_path: &str, local: &str, remote: &str) -> Result<(), String> {
        let _ = Command::new(adb_path)
            .args(["-s", &self.serial, "shell", "rm", "-f", remote])
            .output();

        let out = Command::new(adb_path)
            .args(["-s", &self.serial, "push", local, remote])
            .output()
            .map_err(|e| e.to_string())?;

        if !out.status.success() {
            return Err(format!("Push failed: {}", String::from_utf8_lossy(&out.stderr)));
        }

        let _ = Command::new(adb_path)
            .args(["-s", &self.serial, "shell", "chmod", "755", remote])
            .output();

        Ok(())
    }

    /// Stop video stream
    pub fn stop(&mut self) {
        *self.running.lock().unwrap() = false;
        
        let adb_path = crate::adb::find_adb();
        
        // Kill server on device
        let _ = Command::new(&adb_path)
            .args(["-s", &self.serial, "shell", "pkill", "-9", "-f", "scrcpy"])
            .output();
            
        // Remove reverse
        let _ = Command::new(&adb_path)
            .args(["-s", &self.serial, "reverse", "--remove", &format!("tcp:{}", self.port)])
            .output();

        info!("Video stream stopped");
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }
}

impl Drop for VideoStream {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Stream configuration for frontend
#[derive(Debug, Clone, serde::Serialize)]
pub struct StreamConfig {
    pub ws_url: String,
    pub width: u32,
    pub height: u32,
}

impl VideoStream {
    /// Get WebSocket URL for frontend
    pub fn get_ws_url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.port)
    }
}
