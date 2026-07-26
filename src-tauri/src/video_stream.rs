//! Video streaming via WebSocket using screenrecord
//!
//! Implements low-latency screen streaming using:
//! 1. adb shell screenrecord for h264 capture
//! 2. WebSocket server for streaming to frontend
//! 3. WebCodecs API for browser-side decoding

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::info;

/// Video stream configuration
const DEFAULT_BITRATE: i32 = 8000000;
const DEFAULT_MAX_FPS: i32 = 30;
const DEFAULT_MAX_SIZE: i32 = 1280;

/// Video stream state
pub struct VideoStream {
    pub serial: String,
    pub port: u16,
    pub running: Arc<Mutex<bool>>,
    pub screen_width: Arc<Mutex<u32>>,
    pub screen_height: Arc<Mutex<u32>>,
    process: Option<std::process::Child>,
    ws_handle: Option<thread::JoinHandle<()>>,
}

impl VideoStream {
    pub fn new(serial: String) -> Self {
        Self {
            serial,
            port: 8888,
            running: Arc::new(Mutex::new(false)),
            screen_width: Arc::new(Mutex::new(1080)),
            screen_height: Arc::new(Mutex::new(1920)),
            process: None,
            ws_handle: None,
        }
    }

    /// Start video stream using screenrecord
    pub fn start(&mut self) -> Result<u16, String> {
        let adb_path = crate::adb::find_adb();

        // 1. Get device screen info first
        let screen_info = self.get_screen_info(&adb_path)?;

        // 2. Kill any existing screenrecord
        let _ = Command::new(&adb_path)
            .args([
                "-s",
                &self.serial,
                "shell",
                "pkill",
                "-9",
                "-f",
                "screenrecord",
            ])
            .output();
        thread::sleep(Duration::from_millis(100));

        // 3. Find available port
        let port = self.find_port();
        self.port = port;

        // 4. Set up ADB reverse (device → localhost)
        let _ = Command::new(&adb_path)
            .args([
                "-s",
                &self.serial,
                "reverse",
                "--remove",
                &format!("tcp:{}", port),
            ])
            .output();

        let reverse_out = Command::new(&adb_path)
            .args([
                "-s",
                &self.serial,
                "reverse",
                &format!("tcp:{}", port),
                &format!("tcp:{}", port),
            ])
            .output();

        if let Ok(out) = reverse_out {
            if !out.status.success() {
                return Err(format!(
                    "ADB reverse failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
        }

        // 5. Start screenrecord on device (h264 output)
        let screenrecord_cmd = format!(
            "screenrecord --output-format=h264 --bit-rate={} --max-fps={} --max-size={} --size={}x{} -",
            DEFAULT_BITRATE / 1000, // screenrecord uses kbps
            DEFAULT_MAX_FPS,
            DEFAULT_MAX_SIZE,
            screen_info.0.min(1920),
            screen_info.1.min(1080)
        );

        // Update stored dimensions
        {
            let mut w = self.screen_width.lock().unwrap();
            *w = screen_info.0.min(1920);
        }
        {
            let mut h = self.screen_height.lock().unwrap();
            *h = screen_info.1.min(1080);
        }

        let mut child = Command::new(&adb_path)
            .args(["-s", &self.serial, "shell", &screenrecord_cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start screenrecord: {}", e))?;

        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        self.process = Some(child);

        // 6. Start WebSocket server to stream h264 to browser
        let running = self.running.clone();
        let ws_port = port;

        // Wrap stdout in Arc<Mutex<Box<dyn Read + Send>>> for sharing
        let stdout: Arc<Mutex<Box<dyn Read + Send>>> =
            Arc::new(Mutex::new(Box::new(stdout) as Box<dyn Read + Send>));

        // Mark as running
        *running.lock().unwrap() = true;

        // Start WebSocket server in background thread
        let handle = thread::spawn(move || {
            Self::run_ws_server(ws_port, stdout, running);
        });

        self.ws_handle = Some(handle);

        // Wait a moment for server to start
        thread::sleep(Duration::from_millis(200));

        info!("Video stream started on port {}", port);
        Ok(port)
    }

    fn get_screen_info(&self, adb_path: &str) -> Result<(u32, u32), String> {
        // Get physical display size
        let output = Command::new(adb_path)
            .args(["-s", &self.serial, "shell", "wm", "size"])
            .output()
            .map_err(|e| e.to_string())?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        // Parse output like "Physical size: 1080x1920" or "Override size: 1080x1920"
        for line in output_str.lines() {
            if let Some(size) = line.split(':').next_back() {
                let size = size.trim();
                let parts: Vec<&str> = size.split('x').collect();
                if parts.len() == 2 {
                    if let (Ok(width), Ok(height)) =
                        (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                    {
                        return Ok((width, height));
                    }
                }
            }
        }

        // Default fallback
        Ok((1080, 1920))
    }

    fn find_port(&self) -> u16 {
        for port in [8888, 8889, 8890, 8891, 8892].iter() {
            if TcpStream::connect_timeout(
                &std::net::SocketAddr::from(([127, 0, 0, 1], *port)),
                Duration::from_millis(50),
            )
            .is_err()
            {
                return *port;
            }
        }
        // Fallback to random available port
        if let Ok(listener) = TcpListener::bind("127.0.0.1:0") {
            if let Ok(addr) = listener.local_addr() {
                return addr.port();
            }
        }
        8888
    }

    /// WebSocket server that wraps the h264 stream
    fn run_ws_server(
        port: u16,
        input: Arc<Mutex<Box<dyn Read + Send>>>,
        running: Arc<Mutex<bool>>,
    ) {
        let listener = match TcpListener::bind(format!("127.0.0.1:{}", port)) {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to bind WebSocket server: {}", e);
                return;
            }
        };

        listener.set_nonblocking(true).ok();

        // Wait for client connection
        loop {
            if !*running.lock().unwrap() {
                break;
            }

            match listener.accept() {
                Ok((mut stream, _)) => {
                    info!("WebSocket client connected");
                    let running = running.clone();
                    let input = input.clone();

                    // Simple WebSocket handshake
                    if let Err(e) = Self::ws_handshake(&mut stream) {
                        tracing::error!("WebSocket handshake failed: {}", e);
                        continue;
                    }

                    // Spawn thread to handle this client
                    thread::spawn(move || {
                        let mut buf = [0u8; 65536];
                        while *running.lock().unwrap() {
                            let bytes_read = {
                                let mut input_guard = input.lock().unwrap();
                                input_guard.read(&mut buf)
                            };

                            match bytes_read {
                                Ok(0) => break,
                                Ok(n) => {
                                    // Send as WebSocket binary frame
                                    if let Err(e) = Self::send_ws_frame(&mut stream, 2, &buf[..n]) {
                                        tracing::debug!("Stream write error: {}", e);
                                        break;
                                    }
                                    // Small delay to prevent overwhelming the client
                                    thread::sleep(Duration::from_micros(100));
                                }
                                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                    thread::sleep(Duration::from_millis(10));
                                }
                                Err(e) => {
                                    tracing::error!("Stream read error: {}", e);
                                    break;
                                }
                            }
                        }
                        info!("WebSocket client disconnected");
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    tracing::error!("WebSocket accept error: {}", e);
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    /// Simple WebSocket handshake (HTTP upgrade)
    fn ws_handshake(stream: &mut TcpStream) -> Result<(), String> {
        use std::io::{BufRead, BufReader};

        let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
        let mut request = String::new();

        // Read HTTP request
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).map_err(|e| e.to_string())? == 0 {
                return Err("Connection closed".to_string());
            }
            if line == "\r\n" || line == "\n" {
                break;
            }
            request.push_str(&line);
        }

        // Check for WebSocket upgrade request
        if !request.contains("Upgrade: websocket") {
            return Err("Not a WebSocket request".to_string());
        }

        // Extract Sec-WebSocket-Key
        let key = request
            .lines()
            .filter_map(|line| {
                let line = line.to_lowercase();
                if line.starts_with("sec-websocket-key:") {
                    Some(line.replace("sec-websocket-key:", "").trim().to_string())
                } else {
                    None
                }
            })
            .next()
            .ok_or("No Sec-WebSocket-Key found")?;

        // Generate response key
        let response_key = Self::generate_ws_key(&key);

        // Send WebSocket upgrade response
        let response = format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
            Upgrade: websocket\r\n\
            Connection: Upgrade\r\n\
            Sec-WebSocket-Accept: {}\r\n\
            \r\n",
            response_key
        );

        stream
            .write_all(response.as_bytes())
            .map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Generate WebSocket accept key
    fn generate_ws_key(key: &str) -> String {
        use std::io::Write;

        let mut hasher = std::io::Cursor::new(Vec::new());
        write!(&mut hasher, "{}", key).ok();
        write!(&mut hasher, "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").ok();

        let digest = md5::compute(hasher.into_inner());
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, digest.0)
    }

    /// Send a WebSocket frame
    /// opcode: 2 = binary, 8 = close
    fn send_ws_frame(stream: &mut TcpStream, opcode: u8, payload: &[u8]) -> Result<(), String> {
        let len = payload.len();
        
        // Frame header: FIN=1, opcode, mask=0 (server->client), payload length
        let mut header = Vec::with_capacity(10);
        header.push(0x80 | opcode); // FIN + opcode
        
        if len < 126 {
            header.push(len as u8);
        } else if len < 65536 {
            header.push(126);
            header.push((len >> 8) as u8);
            header.push((len & 0xFF) as u8);
        } else {
            header.push(127);
            for i in (0..8).rev() {
                header.push((len >> (i * 8)) as u8);
            }
        }
        
        stream.write_all(&header).map_err(|e| e.to_string())?;
        stream.write_all(payload).map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())?;
        
        Ok(())
    }

    /// Stop video stream
    pub fn stop(&mut self) {
        *self.running.lock().unwrap() = false;

        // Stop the process
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        let adb_path = crate::adb::find_adb();

        // Kill screenrecord on device
        let _ = Command::new(&adb_path)
            .args([
                "-s",
                &self.serial,
                "shell",
                "pkill",
                "-9",
                "-f",
                "screenrecord",
            ])
            .output();

        // Remove reverse
        let _ = Command::new(&adb_path)
            .args([
                "-s",
                &self.serial,
                "reverse",
                "--remove",
                &format!("tcp:{}", self.port),
            ])
            .output();

        info!("Video stream stopped");
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    /// Get WebSocket URL
    pub fn get_ws_url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.port)
    }

    /// Get screen dimensions
    pub fn get_dimensions(&self) -> (u32, u32) {
        let w = *self.screen_width.lock().unwrap();
        let h = *self.screen_height.lock().unwrap();
        (w, h)
    }
}

impl Drop for VideoStream {
    fn drop(&mut self) {
        self.stop();
    }
}
