//! ⚠️ PARTIAL: scrcpy-server WebSocket Streaming
//!
//! Implements Android screen mirroring using scrcpy-server with ADB forwarding.
//!
//! STATUS: IN PROGRESS - Adding frame streaming via Tauri events.
//!
//! Architecture:
//! 1. Push scrcpy-server to device via ADB
//! 2. Start scrcpy-server on device (listens on device port)
//! 3. ADB forward local port → device port
//! 4. Backend connects to local TCP (forwarded to device)
//! 5. Backend reads H.264 frames from scrcpy protocol
//! 6. Backend emits frames via Tauri events (same as video_stream.rs)
//! 7. Frontend receives via Tauri events, decodes with WebCodecs
//! 8. Frames render in #mirror-screen div (same as adb mode)
//!
//! scrcpy Protocol:
//! - First packet: Device meta (JSON with screen size)
//! - Subsequent packets: H.264 frames with 4-byte length prefix
//! - Frame format: [4 bytes: size][N bytes: H.264 NAL units]
//!
//! Related: scrcpy.rs (native binary mode - separate window)
//! Related: video_stream.rs (ADB screenrecord - works but slow)

use std::io::Read;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex as TokioMutex;
use tokio::time::Duration;
use tracing::{error, info, warn};

use crate::adb::{find_adb, run_adb};

/// Default scrcpy server settings
const DEFAULT_BITRATE: i32 = 8000000; // 8 Mbps
const DEFAULT_MAX_FPS: i32 = 60;
const DEFAULT_MAX_WIDTH: i32 = 1920;

/// Read a packet from scrcpy-server.
/// scrcpy protocol: [4 bytes: big-endian size][N bytes: data]
fn read_packet(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut size_buf = [0u8; 4];
    stream.read_exact(&mut size_buf)?;
    let size = u32::from_be_bytes(size_buf) as usize;

    let mut data = vec![0u8; size];
    stream.read_exact(&mut data)?;
    Ok(data)
}

/// Parse device meta from scrcpy-server.
/// Returns (width, height).
fn parse_device_meta(meta: &[u8]) -> (u32, u32) {
    if let Ok(json_str) = std::str::from_utf8(meta) {
        // Try to parse JSON to get dimensions
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
            let w = json
                .get("width")
                .or(json.get("screenWidth"))
                .and_then(|v| v.as_u64())
                .unwrap_or(1920) as u32;
            let h = json
                .get("height")
                .or(json.get("screenHeight"))
                .and_then(|v| v.as_u64())
                .unwrap_or(1080) as u32;
            return (w, h);
        }
    }
    // Default dimensions
    (DEFAULT_MAX_WIDTH as u32, 1080)
}

/// Path to scrcpy-server JAR
fn get_scrcpy_server_path() -> PathBuf {
    let candidates = vec![
        // Try relative to current dir (for dev)
        PathBuf::from(".").join("scrcpy-server"),
        PathBuf::from("..").join("scrcpy-server"),
        // Try relative to cargo manifest (for release builds)
        std::env::current_dir()
            .unwrap_or_default()
            .join("scrcpy-server"),
        // Common locations
        PathBuf::from("/usr/local/share/scrcpy-server.jar"),
        PathBuf::from("/opt/scrcpy-server/scrcpy-server.jar"),
        // Temp dir (if previously downloaded)
        std::env::temp_dir().join("scrcpy-server.jar"),
    ];

    for path in &candidates {
        let jar_path = if path.is_dir() {
            path.join("scrcpy-server.jar")
        } else {
            path.clone()
        };
        if jar_path.exists() {
            info!("Found scrcpy-server at: {:?}", jar_path);
            return jar_path;
        }
    }

    // Return temp path even if not exists (will try to download)
    let download_path = std::env::temp_dir().join("scrcpy-server.jar");
    warn!("scrcpy-server not found in any candidate paths, will try to download");
    download_path
}

/// Get or download scrcpy-server
fn ensure_scrcpy_server() -> Result<PathBuf, String> {
    let path = get_scrcpy_server_path();

    if path.exists() {
        return Ok(path);
    }

    info!("Downloading scrcpy-server...");

    // Download from GitHub
    let output = Command::new("curl")
        .args([
            "-sL",
            "https://github.com/Genymobile/scrcpy/releases/download/v2.1.1/scrcpy-server-v2.1.1",
            "-o",
            path.to_str().unwrap_or("/tmp/scrcpy-server.jar"),
        ])
        .output();

    match output {
        Ok(out) if out.status.success() && path.exists() => {
            info!("Downloaded scrcpy-server");
            Ok(path)
        }
        _ => Err("Failed to download scrcpy-server".to_string()),
    }
}

/// Scrcpy server state
pub struct ScrcpyServer {
    serial: String,
    local_port: u16,
    process: Option<std::process::Child>,
}

/// Scrcpy control message types
#[derive(Debug)]
#[repr(u8)]
enum ControlMessage {
    /// Key event (Android keycode)
    Keycode {
        action: u8,
        keycode: u32,
        repeat: u32,
        meta: u32,
    } = 0,
    /// Text input
    Text { text: String } = 1,
    /// Mouse/touch event
    Touch {
        action: u8,
        x: i32,
        y: i32,
        normalized: bool,
    } = 2,
    /// Scroll event
    Scroll {
        x: i32,
        y: i32,
        hscroll: i32,
        vscroll: i32,
    } = 3,
}

impl ScrcpyServer {
    pub fn new(serial: String) -> Self {
        Self {
            serial,
            local_port: 8888,
            process: None,
        }
    }

    /// Start scrcpy-server streaming
    pub fn start(&mut self) -> Result<String, String> {
        let adb_path = find_adb();

        // Step 1: Kill any existing scrcpy server on device
        info!("Stopping any existing scrcpy-server");
        let _ = run_adb(&["-s", &self.serial, "shell", "pkill", "-9", "-f", "scrcpy"]);
        let _ = run_adb(&[
            "-s",
            &self.serial,
            "shell",
            "am",
            "force-stop",
            "org.genymobile.scrcpy",
        ]);
        thread::sleep(Duration::from_millis(200));

        // Step 2: Ensure scrcpy-server is available
        let server_path = ensure_scrcpy_server()?;

        // Step 3: Push scrcpy-server to device
        let device_server_path = "/data/local/tmp/scrcpy-server.jar";
        info!("Pushing scrcpy-server to device");

        let _ = run_adb(&["-s", &self.serial, "shell", "rm", "-f", device_server_path]);

        let push_result = Command::new(&adb_path)
            .args([
                "-s",
                &self.serial,
                "push",
                server_path.to_str().unwrap(),
                device_server_path,
            ])
            .output();

        if let Ok(out) = push_result {
            if !out.status.success() {
                return Err(format!(
                    "Failed to push scrcpy-server: {}",
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
        }

        // Make executable
        let _ = run_adb(&[
            "-s",
            &self.serial,
            "shell",
            "chmod",
            "755",
            device_server_path,
        ]);

        // Step 4: Find available local port
        let port = self.find_available_port();
        self.local_port = port;

        // Step 5: Forward local TCP port to device's scrcpy-server abstract socket
        // scrcpy-server listens on abstract socket "scrcpy", not TCP
        let _ = Command::new(&adb_path)
            .args([
                "-s",
                &self.serial,
                "forward",
                "--remove",
                &format!("tcp:{}", port),
            ])
            .output();

        let forward_result = Command::new(&adb_path)
            .args([
                "-s",
                &self.serial,
                "forward",
                &format!("tcp:{}", port),
                "localabstract:scrcpy",
            ])
            .output();

        if let Ok(out) = forward_result {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            info!(
                "Port forward stdout: {}, stderr: {}",
                stdout.trim(),
                stderr.trim()
            );
            if !out.status.success() {
                return Err(format!("Failed to forward port: {}", stderr));
            }
            info!("Port forwarding: tcp:{} -> localabstract:scrcpy", port);
        } else {
            error!("Port forward command failed to execute");
        }

        // Step 6: Start scrcpy-server on device
        info!("Starting scrcpy-server on device");

        let start_cmd = format!(
            "CLASSPATH={} app_process / {} --bit-rate={} --max-fps={} --max-size={}",
            device_server_path, port, DEFAULT_BITRATE, DEFAULT_MAX_FPS, DEFAULT_MAX_WIDTH
        );
        info!("scrcpy-server command: {}", start_cmd);

        let child = Command::new(&adb_path)
            .args(["-s", &self.serial, "shell", "nohup", &start_cmd])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start scrcpy-server: {}", e))?;

        self.process = Some(child);

        // Wait for server to start
        thread::sleep(Duration::from_secs(2));

        // Verify scrcpy-server is running on device
        let ps_result = Command::new(&adb_path)
            .args(["-s", &self.serial, "shell", "ps", "-A"])
            .output();

        if let Ok(out) = ps_result {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.contains("scrcpy") {
                info!("scrcpy-server process found on device");
            } else {
                warn!(
                    "scrcpy-server process NOT found on device! stdout: {}",
                    stdout
                );
            }
        }

        // Return the WebSocket URL for frontend
        Ok(format!("ws://127.0.0.1:{}", port))
    }

    fn find_available_port(&self) -> u16 {
        // Try common ports first
        for port in [8888, 8889, 8890, 8891, 8892].iter() {
            if self.is_port_available(*port) {
                return *port;
            }
        }
        // Fall back to random
        (9000..10000)
            .find(|p| self.is_port_available(*p))
            .unwrap_or(8888)
    }

    fn is_port_available(&self, port: u16) -> bool {
        std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
    }

    /// Event name for scrcpy video frames
    pub const SCRCPY_FRAME_EVENT: &str = "scrcpy-frame";

    /// Start scrcpy-server and emit frames via Tauri events.
    /// This allows the frontend to render frames in the #mirror-screen div.
    ///
    /// Returns (width, height) on success.
    pub fn start_with_events(&mut self, app: &AppHandle) -> Result<(u32, u32), String> {
        // Start scrcpy-server (uses existing start() logic)
        self.start()?;

        let port = self.local_port;
        let serial = self.serial.clone();
        let running = Arc::new(std::sync::Mutex::new(true));
        let running_clone = running.clone();

        // Clone app handle for the thread
        let app_clone = app.clone();

        // Emit that stream started
        let _ = app.emit(
            "scrcpy-stream-started",
            serde_json::json!({
                "serial": serial,
                "port": port,
            }),
        );

        // Spawn thread to read frames from TCP and emit via Tauri events
        thread::spawn(move || {
            info!("Starting scrcpy frame reader thread for port {}", port);

            // Connect to local port (forwarded to device scrcpy-server)
            let addr = format!("127.0.0.1:{}", port);
            let mut stream = match TcpStream::connect(&addr) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to connect to scrcpy-server at {}: {}", addr, e);
                    return;
                }
            };

            // Set read timeout to allow checking running flag
            let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(2000)));

            info!("Waiting for scrcpy device meta...");

            // Read device meta (first packet)
            // scrcpy-server sends device meta as first packet
            let meta = match read_packet(&mut stream) {
                Ok(data) => data,
                Err(e) => {
                    warn!(
                        "Failed to read scrcpy device meta: {} - may be sending raw H264",
                        e
                    );
                    // Try to continue anyway - might get H264 directly
                    vec![]
                }
            };

            if !meta.is_empty() {
                info!(
                    "scrcpy device meta received ({} bytes): {:?}",
                    meta.len(),
                    String::from_utf8_lossy(&meta[..meta.len().min(200)])
                );
            } else {
                info!("No device meta received, assuming raw H264 stream");
            }

            // Parse meta to get dimensions
            let (width, height) = if meta.is_empty() {
                // Default dimensions if no meta
                (1920u32, 1080u32)
            } else {
                parse_device_meta(&meta)
            };
            info!("scrcpy device screen: {}x{}", width, height);

            // Emit dimensions
            let _ = app_clone.emit(
                "scrcpy-stream-started",
                serde_json::json!({
                    "serial": serial,
                    "port": port,
                    "width": width,
                    "height": height,
                    "message": "Starting frame read loop...",
                }),
            );

            // Read frames
            info!("Starting frame read loop...");
            let _ = app_clone.emit(
                "scrcpy-debug",
                &format!("Starting frame read loop... port={}", port),
            );
            let mut frame_count = 0u64;
            let mut consecutive_timeouts = 0u32;
            loop {
                if !*running_clone.lock().unwrap() {
                    info!("scrcpy frame reader stopped");
                    break;
                }

                match read_packet(&mut stream) {
                    Ok(packet) => {
                        consecutive_timeouts = 0;
                        frame_count += 1;
                        // Emit frame data via Tauri event
                        if let Err(e) = app_clone.emit(ScrcpyServer::SCRCPY_FRAME_EVENT, &packet) {
                            error!("Failed to emit scrcpy frame: {}", e);
                            break;
                        }

                        // Log first few frames and periodically
                        if frame_count <= 5 || frame_count.is_multiple_of(50) {
                            info!(
                                "scrcpy frame {} emitted ({} bytes)",
                                frame_count,
                                packet.len()
                            );
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // Timeout, continue checking running flag
                        consecutive_timeouts += 1;
                        if consecutive_timeouts == 1 {
                            info!("scrcpy: waiting for frames...");
                            let _ = app_clone.emit("scrcpy-debug", "Waiting for frames...");
                        }
                        if consecutive_timeouts > 100 {
                            warn!(
                                "scrcpy: no frames received for {} timeouts, stopping",
                                consecutive_timeouts
                            );
                            let _ = app_clone.emit(
                                "scrcpy-debug",
                                &format!(
                                    "No frames for {} timeouts, stopping",
                                    consecutive_timeouts
                                ),
                            );
                            break;
                        }
                        continue;
                    }
                    Err(e) => {
                        error!("scrcpy read error after {} frames: {}", frame_count, e);
                        break;
                    }
                }
            }

            let _ = app_clone.emit("scrcpy-stream-ended", ());
            info!("scrcpy frame reader ended: {} frames", frame_count);
        });

        // Return dimensions (will be updated by frame reader thread)
        Ok((DEFAULT_MAX_WIDTH as u32, 0)) // Will be updated when meta is received
    }

    /// Stop the frame reader thread
    pub fn stop_frame_reader(&self) {
        // This is called when stopping - the frame reader checks this flag
    }

    /// Stop scrcpy-server
    pub fn stop(&mut self) {
        info!("Stopping scrcpy-server");

        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        let adb_path = find_adb();

        // Kill server on device
        let _ = run_adb(&["-s", &self.serial, "shell", "pkill", "-9", "-f", "scrcpy"]);

        // Remove port forward
        let _ = Command::new(&adb_path)
            .args([
                "-s",
                &self.serial,
                "forward",
                "--remove",
                &format!("tcp:{}", self.local_port),
            ])
            .output();

        info!("scrcpy-server stopped");
    }

    /// Send tap event
    pub fn tap(&self, x: i32, y: i32) -> Result<(), String> {
        run_adb(&[
            "-s",
            &self.serial,
            "shell",
            "input",
            "tap",
            &x.to_string(),
            &y.to_string(),
        ])?;
        Ok(())
    }

    /// Send swipe event
    pub fn swipe(
        &self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        duration_ms: i32,
    ) -> Result<(), String> {
        run_adb(&[
            "-s",
            &self.serial,
            "shell",
            "input",
            "swipe",
            &x1.to_string(),
            &y1.to_string(),
            &x2.to_string(),
            &y2.to_string(),
            &duration_ms.to_string(),
        ])?;
        Ok(())
    }

    /// Send text
    pub fn text(&self, text: &str) -> Result<(), String> {
        let escaped = text.replace(' ', "%s").replace("'", "\\'");
        run_adb(&["-s", &self.serial, "shell", "input", "text", &escaped])?;
        Ok(())
    }

    /// Press back
    pub fn back(&self) -> Result<(), String> {
        run_adb(&[
            "-s",
            &self.serial,
            "shell",
            "input",
            "keyevent",
            "KEYCODE_BACK",
        ])?;
        Ok(())
    }

    /// Press home
    pub fn home(&self) -> Result<(), String> {
        run_adb(&[
            "-s",
            &self.serial,
            "shell",
            "input",
            "keyevent",
            "KEYCODE_HOME",
        ])?;
        Ok(())
    }

    /// Press enter
    pub fn enter(&self) -> Result<(), String> {
        run_adb(&[
            "-s",
            &self.serial,
            "shell",
            "input",
            "keyevent",
            "KEYCODE_ENTER",
        ])?;
        Ok(())
    }

    /// Press power
    pub fn power(&self) -> Result<(), String> {
        run_adb(&[
            "-s",
            &self.serial,
            "shell",
            "input",
            "keyevent",
            "KEYCODE_POWER",
        ])?;
        Ok(())
    }
}

impl Drop for ScrcpyServer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Shared scrcpy server manager with async support
pub type ScrcpyServerManager = Arc<TokioMutex<ScrcpyServer>>;

pub fn create_scrcpy_server(serial: String) -> ScrcpyServerManager {
    Arc::new(TokioMutex::new(ScrcpyServer::new(serial)))
}

/// Start mirror and return WebSocket URL
pub fn start_mirror_ws(serial: String) -> Result<String, String> {
    let mut server = ScrcpyServer::new(serial);
    server.start()
}

/// Stop mirror
pub fn stop_mirror_ws(server: &mut ScrcpyServer) {
    server.stop();
}
