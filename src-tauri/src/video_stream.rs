//! Video streaming via Tauri events using screenrecord
//!
//! Implements low-latency screen streaming using:
//! 1. adb shell screenrecord for h264 capture
//! 2. Tauri events for backend→frontend IPC (no WebSocket needed)
//! 3. WebCodecs API for browser-side decoding
//!
//! This avoids the macOS sandbox issue with TCP sockets.

use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tracing::{error, info};

/// Event name used to emit video frames to the frontend.
pub const VIDEO_FRAME_EVENT: &str = "video-frame";

/// Video stream configuration
const DEFAULT_BITRATE: i32 = 8_000_000;
const DEFAULT_MAX_FPS: i32 = 30;
const DEFAULT_MAX_SIZE: i32 = 1280;

/// Video stream state
pub struct VideoStream {
    pub serial: String,
    pub running: Arc<Mutex<bool>>,
    pub screen_width: Arc<Mutex<u32>>,
    pub screen_height: Arc<Mutex<u32>>,
    process: Option<std::process::Child>,
    stream_handle: Option<thread::JoinHandle<()>>,
}

impl VideoStream {
    pub fn new(serial: String) -> Self {
        Self {
            serial,
            running: Arc::new(Mutex::new(false)),
            screen_width: Arc::new(Mutex::new(1080)),
            screen_height: Arc::new(Mutex::new(1920)),
            process: None,
            stream_handle: None,
        }
    }

    /// Start video stream using screenrecord + Tauri events.
    ///
    /// Returns the screen dimensions (width, height) on success.
    pub fn start(&mut self, app: &AppHandle) -> Result<(u32, u32), String> {
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

        // 3. Update stored dimensions (cap to 1920x1080 for performance)
        let width = screen_info.0.min(1920);
        let height = screen_info.1.min(1080);
        {
            let mut w = self.screen_width.lock().unwrap();
            *w = width;
        }
        {
            let mut h = self.screen_height.lock().unwrap();
            *h = height;
        }

        // 4. Start screenrecord on device (h264 output)
        let screenrecord_cmd = format!(
            "screenrecord --output-format=h264 --bit-rate={} --max-fps={} --max-size={} --size={}x{} -",
            DEFAULT_BITRATE / 1000, // screenrecord uses kbps
            DEFAULT_MAX_FPS,
            DEFAULT_MAX_SIZE,
            width,
            height
        );

        let mut child = Command::new(&adb_path)
            .args(["-s", &self.serial, "shell", &screenrecord_cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start screenrecord: {}", e))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture stdout".to_string())?;
        self.process = Some(child);
        let mut stdout = stdout;

        // 5. Mark as running
        *self.running.lock().unwrap() = true;

        // 6. Start thread that reads H264 frames and emits via Tauri events
        let running = self.running.clone();
        let app_handle = app.clone();
        let serial = self.serial.clone();
        let width_clone = width;
        let height_clone = height;

        // Emit initial metadata event so frontend knows dimensions
        let _ = app_handle.emit(
            "video-stream-started",
            serde_json::json!({
                "width": width_clone,
                "height": height_clone,
                "serial": serial,
            }),
        );

        let handle = thread::spawn(move || {
            info!("Video stream thread started");
            let mut buf = vec![0u8; 65536];
            let mut frame_count = 0u64;
            let mut total_bytes = 0u64;

            while *running.lock().unwrap() {
                match stdout.read(&mut buf) {
                    Ok(0) => {
                        info!(
                            "Video stream EOF ({} frames, {} bytes)",
                            frame_count, total_bytes
                        );
                        break;
                    }
                    Ok(n) => {
                        frame_count += 1;
                        total_bytes += n as u64;
                        // Emit frame via Tauri event
                        if let Err(e) = app_handle.emit(VIDEO_FRAME_EVENT, &buf[..n]) {
                            error!("Failed to emit video frame: {}", e);
                            break;
                        }
                        if frame_count <= 3 || frame_count.is_multiple_of(100) {
                            info!(
                                "Video frame {} ({} bytes, total {} MB)",
                                frame_count,
                                n,
                                total_bytes / 1_048_576
                            );
                        }
                    }
                    Err(e) => {
                        error!("Stream read error after {} frames: {}", frame_count, e);
                        break;
                    }
                }
            }

            info!(
                "Video stream ended: {} frames, {} MB total",
                frame_count,
                total_bytes / 1_048_576
            );
            let _ = app_handle.emit("video-stream-ended", ());
        });

        self.stream_handle = Some(handle);
        info!(
            "Video stream started for {} ({}x{})",
            self.serial, width, height
        );

        Ok((width, height))
    }

    /// Stop the video stream
    pub fn stop(&mut self) {
        *self.running.lock().unwrap() = false;

        // Kill screenrecord
        if let Some(mut child) = self.process.take() {
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
            let _ = child.kill();
            let _ = child.wait();
        }

        // Wait for stream thread to finish (it should exit quickly)
        if let Some(handle) = self.stream_handle.take() {
            // Don't block forever - thread checks running flag periodically
            let _ = handle.join();
        }

        info!("Video stream stopped for {}", self.serial);
    }

    /// Check if stream is running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    /// Get screen dimensions
    pub fn get_dimensions(&self) -> (u32, u32) {
        (
            *self.screen_width.lock().unwrap(),
            *self.screen_height.lock().unwrap(),
        )
    }

    /// Get device screen info via ADB
    fn get_screen_info(&self, adb_path: &str) -> Result<(u32, u32), String> {
        let out = Command::new(adb_path)
            .args(["-s", &self.serial, "shell", "wm", "size"])
            .output()
            .map_err(|e| format!("Failed to get screen size: {}", e))?;

        let stdout = String::from_utf8_lossy(&out.stdout);
        // Parse "Physical size: 1080x1920" or "Override size: 1080x1920"
        let parts: Vec<&str> = stdout.split_whitespace().collect();
        if let Some(size_str) = parts.last() {
            let dims: Vec<&str> = size_str.split('x').collect();
            if dims.len() == 2 {
                if let (Ok(w), Ok(h)) = (dims[0].parse::<u32>(), dims[1].parse::<u32>()) {
                    return Ok((w, h));
                }
            }
        }
        // Default to common phone size
        Ok((1080, 1920))
    }
}
