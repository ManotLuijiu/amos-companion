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
const SCRCPY_SERVER_VERSION: &str = "4.1"; // Must match the scrcpy-server.jar version
const DEFAULT_BITRATE: i32 = 4_000_000; // 4 Mbps - reduced for smoother rendering
const DEFAULT_MAX_FPS: i32 = 30; // 30 fps - halves decode/render load
const DEFAULT_MAX_WIDTH: i32 = 1920;

/// Frame payload for frontend (matches frontend contract: {bytes: [...], key: true|false})
#[derive(serde::Serialize)]
struct ScrcpyFramePayload<'a> {
    bytes: &'a [u8],
    key: bool,
}

/// Path to scrcpy-server JAR
fn get_scrcpy_server_path() -> PathBuf {
    // Collect all candidate paths
    let mut candidates: Vec<PathBuf> = Vec::new();

    // 1. Bundled resource (from tauri.conf.json resources)
    //    In dev: src-tauri/scrcpy-server/scrcpy-server.jar
    //    In release: relative to executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent().map(|p| p.to_path_buf()) {
            info!("DEBUG: exe={:?}, exe_dir={:?}", exe, exe_dir);
            candidates.push(exe_dir.join("scrcpy-server.jar"));
            candidates.push(exe_dir.join("resources/scrcpy-server.jar"));

            // 3. macOS app bundle: Contents/MacOS/amos-companion -> Contents/Resources
            if let Some(contents) = exe_dir.parent() {
                info!("DEBUG: contents={:?}", contents);
                candidates.push(contents.join("Resources").join("scrcpy-server.jar"));
            }
        }
    } else {
        warn!("DEBUG: current_exe() returned None");
    }

    // 2. Dev paths (relative to cargo manifest)
    let cargo_dir = std::env::current_dir().unwrap_or_default();
    candidates.push(
        cargo_dir
            .join("src-tauri")
            .join("scrcpy-server")
            .join("scrcpy-server.jar"),
    );
    candidates.push(cargo_dir.join("scrcpy-server").join("scrcpy-server.jar"));

    // 4. Common install locations
    candidates.push(PathBuf::from("/usr/local/share/scrcpy-server.jar"));
    candidates.push(PathBuf::from("/opt/scrcpy-server/scrcpy-server.jar"));

    // 5. Temp dir (if previously downloaded)
    candidates.push(std::env::temp_dir().join("scrcpy-server.jar"));

    info!(
        "DEBUG: Checking {} candidate paths for scrcpy-server.jar",
        candidates.len()
    );
    for jar_path in &candidates {
        let exists = jar_path.exists();
        info!("DEBUG: path={:?} exists={}", jar_path, exists);
        if exists {
            info!("Found scrcpy-server at: {:?}", jar_path);
            return jar_path.clone();
        }
    }

    // Return temp path even if not exists (will try to download)
    let download_path = std::env::temp_dir().join("scrcpy-server.jar");
    warn!(
        "scrcpy-server not found in {:?}, will try to download",
        candidates
    );
    download_path
}

/// Get or download scrcpy-server
fn ensure_scrcpy_server() -> Result<PathBuf, String> {
    let path = get_scrcpy_server_path();

    if path.exists() {
        return Ok(path);
    }

    info!("Downloading scrcpy-server v{}...", SCRCPY_SERVER_VERSION);

    // URL MUST match SCRCPY_SERVER_VERSION exactly. v4.x artifacts have no
    // .jar extension in the GitHub release, so the trailing `.jar` is NOT
    // appended on the server side. Older (e.g. v2.x) releases DO have `.jar`.
    let download_url = format!(
        "https://github.com/Genymobile/scrcpy/releases/download/v{}/scrcpy-server-v{}",
        SCRCPY_SERVER_VERSION, SCRCPY_SERVER_VERSION
    );

    let output = Command::new("curl")
        .args([
            "-sL",
            &download_url,
            "-o",
            path.to_str().unwrap_or("/tmp/scrcpy-server.jar"),
        ])
        .output();

    match output {
        Ok(out) if out.status.success() && path.exists() => {
            info!("Downloaded scrcpy-server v{}", SCRCPY_SERVER_VERSION);
            Ok(path)
        }
        _ => {
            let stderr = match output {
                Ok(out) => String::from_utf8_lossy(&out.stderr).to_string(),
                Err(e) => e.to_string(),
            };
            Err(format!(
                "Failed to download scrcpy-server v{} from {}: {}",
                SCRCPY_SERVER_VERSION, download_url, stderr
            ))
        }
    }
}

/// Scrcpy server state
pub struct ScrcpyServer {
    serial: String,
    local_port: u16,
    process: Option<std::process::Child>,
    debug_info: String,
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
            debug_info: String::new(),
        }
    }

    /// Get the local port used for port forwarding
    pub fn get_local_port(&self) -> u16 {
        self.local_port
    }

    /// Get debug info from start()
    pub fn get_debug_info(&self) -> &str {
        &self.debug_info
    }

    /// Start scrcpy-server streaming
    pub fn start(&mut self) -> Result<bool, String> {
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
            // Store for later emission
            self.debug_info = format!(
                "Port forward result: stdout='{}', stderr='{}'",
                stdout.trim(),
                stderr.trim()
            );
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
            self.debug_info = "Port forward command failed to execute".to_string();
        }

        // Step 6: Start scrcpy-server on device
        // FIXED: tunnel_forward=true with adb forward (required combination)
        // audio=false, control=false since we only read video
        info!("Starting scrcpy-server on device");

        let start_cmd = format!(
            "CLASSPATH={} app_process / com.genymobile.scrcpy.Server {} tunnel_forward=true audio=false control=false video_bit_rate={} max_fps={} max_size={}",
            device_server_path,
            SCRCPY_SERVER_VERSION,
            DEFAULT_BITRATE,
            DEFAULT_MAX_FPS,
            DEFAULT_MAX_WIDTH
        );
        info!("scrcpy-server command: {}", start_cmd);
        self.debug_info += &format!("\nscrcpy-server command: {}", start_cmd);

        // Launch app_process directly (NO `nohup`). With `nohup CLASSPATH=… app_process`,
        // the shell treats `CLASSPATH=…` as a literal argument to nohup (not an env
        // assignment), so nohup tries to exec the string "CLASSPATH=…" as a program
        // and fails with "exec CLASSPATH=…: No such file or directory" — app_process
        // never starts. Without nohup, `CLASSPATH=…` is in prefix position, the shell
        // applies it to app_process, and the server runs (held alive by this spawned
        // adb child; stop() kills the child → adb disconnect → server exits, then the
        // pkill below cleans up any survivor).
        let child = Command::new(&adb_path)
            .args([
                "-s",
                &self.serial,
                "shell",
                &start_cmd,
                ">/dev/null",
                "2>/data/local/tmp/scrcpy-server.err",
            ])
            .stdout(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start scrcpy-server: {}", e))?;

        self.process = Some(child);

        // Wait for server to start (scrcpy-server takes time to initialize)
        thread::sleep(Duration::from_secs(3));

        // Informational: check if scrcpy-server process is running on device
        let ps_result = Command::new(&adb_path)
            .args(["-s", &self.serial, "shell", "ps", "-A"])
            .output();

        if let Ok(out) = ps_result {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.contains("app_process") && stdout.contains("scrcpy") {
                info!("scrcpy-server process found on device (diagnostic)");
                self.debug_info += "\nscrcpy-server process found (diagnostic)";
            }
        }

        // Always return Ok(true) - TCP connection is the real liveness proof
        // (checked in start_with_events)
        Ok(true)
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
    /// Implements scrcpy 4.x wire protocol.
    ///
    /// Returns (width, height, running) on success.
    pub fn start_with_events(&mut self, app: &AppHandle) -> Result<(u32, u32, bool), String> {
        // Start scrcpy-server
        self.start()?;

        let port = self.local_port;
        let serial = self.serial.clone();
        let running = Arc::new(std::sync::Mutex::new(true));
        let running_clone = running.clone();
        let app_clone = app.clone();

        // Retry loop: connect + dummy byte is atomic readiness check
        let addr: std::net::SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
        let mut stream: Option<TcpStream> = None;
        let mut last_err = String::new();

        for attempt in 1..=15u32 {
            if !*running.lock().unwrap() {
                break;
            }
            match TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(2)) {
                Ok(mut s) => {
                    let mut dummy = [0u8; 1];
                    match std::io::Read::read_exact(&mut s, &mut dummy) {
                        Ok(()) => {
                            info!(
                                "scrcpy handshake ok on attempt {} (dummy byte read)",
                                attempt
                            );
                            stream = Some(s);
                            break;
                        }
                        Err(e) => {
                            last_err = format!("dummy byte read on attempt {}: {}", attempt, e);
                            warn!("{}", last_err);
                            drop(s);
                            thread::sleep(std::time::Duration::from_millis(300));
                        }
                    }
                }
                Err(e) => {
                    last_err = format!("connect on attempt {}: {}", attempt, e);
                    warn!("{}", last_err);
                    thread::sleep(std::time::Duration::from_millis(300));
                }
            }
        }

        let mut stream = match stream {
            Some(s) => s,
            None => {
                // Read stderr for diagnostics
                if let Ok(out) = Command::new(find_adb())
                    .args([
                        "-s",
                        &self.serial,
                        "shell",
                        "cat",
                        "/data/local/tmp/scrcpy-server.err",
                    ])
                    .output()
                {
                    let err = String::from_utf8_lossy(&out.stdout);
                    if !err.is_empty() {
                        error!("scrcpy-server stderr:\n{}", err);
                    }
                }
                return Err(format!(
                    "Failed to complete scrcpy handshake after retries: {}",
                    last_err
                ));
            }
        };

        stream.set_nodelay(true).ok();

        // Read device meta: a FIXED 64-byte field (device name, NUL-padded).
        // scrcpy's DesktopConnection.sendDeviceMeta() writes a fixed-size buffer
        // (DEVICE_NAME_FIELD_LENGTH = 64) with NO length prefix. The previous code
        // read a u16 length, which grabbed the first 2 chars of the name ("SM" =
        // 0x534d = 21325) and then blocked trying to read 21325 bytes.
        const DEVICE_NAME_FIELD_LENGTH: usize = 64;
        let mut meta_buf = [0u8; DEVICE_NAME_FIELD_LENGTH];
        std::io::Read::read_exact(&mut stream, &mut meta_buf)
            .map_err(|e| format!("Failed to read device meta: {}", e))?;
        let name_end = meta_buf
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(DEVICE_NAME_FIELD_LENGTH);
        let device_name = String::from_utf8_lossy(&meta_buf[..name_end]);
        info!("Device name: {}", device_name);

        // Read codec id (4 ASCII bytes, e.g. "h264")
        let mut codec_buf = [0u8; 4];
        std::io::Read::read_exact(&mut stream, &mut codec_buf)
            .map_err(|e| format!("Failed to read codec id: {}", e))?;
        let codec = String::from_utf8_lossy(&codec_buf);
        info!("Codec: {}", codec);

        // Read session packet (12 bytes, flag 0x80) for width/height
        let mut session_buf = [0u8; 12];
        std::io::Read::read_exact(&mut stream, &mut session_buf)
            .map_err(|e| format!("Failed to read session packet: {}", e))?;

        let mut width: u32 = DEFAULT_MAX_WIDTH as u32;
        let mut height: u32 = 1080;

        if session_buf[0] & 0x80 != 0 {
            width = u32::from_be_bytes([
                session_buf[4],
                session_buf[5],
                session_buf[6],
                session_buf[7],
            ]);
            height = u32::from_be_bytes([
                session_buf[8],
                session_buf[9],
                session_buf[10],
                session_buf[11],
            ]);
            info!("Session: {}x{}", width, height);
        } else {
            warn!("First packet was not session packet, reading media...");
        }

        // Emit stream started with dimensions
        let _ = app.emit(
            "scrcpy-stream-started",
            serde_json::json!({
                "serial": serial,
                "port": port,
                "width": width,
                "height": height,
                "message": "scrcpy 4.x protocol handshake complete",
            }),
        );

        info!("Starting scrcpy media loop ({}x{})", width, height);

        // Spawn thread for ongoing media stream (blocking reads)
        thread::spawn(move || {
            let mut stream = stream;

            let mut pending_config: Option<Vec<u8>> = None;
            let mut frame_count: u64 = 0;

            loop {
                if !*running_clone.lock().unwrap() {
                    info!("scrcpy frame reader stopped");
                    break;
                }

                let mut hdr = [0u8; 12];
                match std::io::Read::read_exact(&mut stream, &mut hdr) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                        info!("scrcpy server disconnected");
                        break;
                    }
                    Err(e) => {
                        error!("scrcpy header read: {}", e);
                        break;
                    }
                }

                let b0 = hdr[0];
                if b0 & 0x80 != 0 {
                    // Session packet (rotation change) - update dimensions
                    let w = u32::from_be_bytes([hdr[4], hdr[5], hdr[6], hdr[7]]);
                    let h = u32::from_be_bytes([hdr[8], hdr[9], hdr[10], hdr[11]]);
                    let _ = app_clone.emit(
                        "scrcpy-stream-started",
                        serde_json::json!({
                            "serial": serial,
                            "width": w,
                            "height": h,
                        }),
                    );
                    continue;
                }

                // Media packet
                let packet_size = u32::from_be_bytes([hdr[8], hdr[9], hdr[10], hdr[11]]) as usize;
                let is_key = (b0 & 0x20) != 0;
                let is_config = (b0 & 0x40) != 0;

                let mut payload = vec![0u8; packet_size];
                if let Err(e) = std::io::Read::read_exact(&mut stream, &mut payload) {
                    error!("scrcpy body read: {}", e);
                    break;
                }

                if is_config {
                    // SPS/PPS config packet - stash for first keyframe
                    pending_config = Some(payload);
                    continue;
                }

                // Emit frame with correct payload shape {bytes: [...], key: true|false}
                let (bytes, key) = if is_key {
                    let combined = match pending_config.take() {
                        Some(cfg) => {
                            let mut v = cfg;
                            v.extend_from_slice(&payload);
                            v
                        }
                        None => payload,
                    };
                    (combined, true)
                } else {
                    (payload, false)
                };

                let _ = app_clone.emit(
                    ScrcpyServer::SCRCPY_FRAME_EVENT,
                    &ScrcpyFramePayload { bytes: &bytes, key },
                );

                frame_count += 1;
                if frame_count <= 3 || frame_count.is_multiple_of(50) {
                    info!(
                        "scrcpy frame {} ({} bytes, key={})",
                        frame_count,
                        bytes.len(),
                        key
                    );
                }
            }

            let _ = app_clone.emit("scrcpy-stream-ended", ());
            info!("scrcpy media loop ended: {} frames", frame_count);
        });

        Ok((width, height, true))
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

/// Start mirror (legacy compatibility wrapper)
/// Returns the local WebSocket URL on success, an error message on failure.
pub fn start_mirror_ws(serial: String) -> Result<String, String> {
    let mut server = ScrcpyServer::new(serial);
    if server.start()? {
        Ok(format!("ws://127.0.0.1:{}", server.local_port))
    } else {
        Err("scrcpy-server process not alive on device".to_string())
    }
}

/// Stop mirror
pub fn stop_mirror_ws(server: &mut ScrcpyServer) {
    server.stop();
}
