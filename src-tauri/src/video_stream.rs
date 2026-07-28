//! ADB Video Streaming via screenrecord (BUILT-IN MIRROR)
//!
//! ✅ STATUS: WORKING (but limited performance due to ADB overhead)
//! This is the built-in mirror mode that streams video via:
//! 1. `adb shell screenrecord` for H.264 capture
//! 2. Tauri events for backend→frontend IPC
//! 3. WebCodecs API for browser-side decoding
//!
//! PERFORMANCE NOTE: ADB adds significant latency (~100-300ms per frame).
//! For better performance, use scrcpy mode (scrcpy_server.rs) instead.
//!
//! H.264 NAL Unit Handling:
//! The raw H.264 stream from screenrecord must be parsed at NAL unit boundaries.
//! NAL units start with one of these start codes:
//!   - 0x00 0x00 0x00 0x01 (4-byte start code)
//!   - 0x00 0x00 0x01 (3-byte start code)
//! We buffer incoming bytes and extract complete NAL units before emitting.

use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tracing::{error, info, warn};

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
    /// Buffered data from screenrecord for NAL unit extraction
    nal_buffer: Vec<u8>,
}

/// Find all NAL unit start positions in the buffer.
/// Returns vector of (start_pos, end_pos) tuples.
fn find_nal_units(data: &[u8]) -> Vec<(usize, usize)> {
    let mut units = Vec::new();
    let mut i = 0;

    while i < data.len() {
        // Look for start code: 0x00 0x00 [0x00] 0x01
        if i + 3 < data.len() && data[i] == 0x00 && data[i + 1] == 0x00 {
            let start_code_len = if i + 4 < data.len() && data[i + 2] == 0x00 && data[i + 3] == 0x01
            {
                // 4-byte start code: 0x00 0x00 0x00 0x01
                4
            } else if data[i + 2] == 0x01 {
                // 3-byte start code: 0x00 0x00 0x01
                3
            } else {
                i += 1;
                continue;
            };

            // Find the end of this NAL unit (next start code or end of data)
            let nal_start = i + start_code_len;
            let mut nal_end = data.len();

            // Search for next start code
            let mut j = nal_start + 1;
            while j < data.len() - 3 {
                if data[j] == 0x00 && data[j + 1] == 0x00 {
                    if data[j + 2] == 0x00 && data[j + 3] == 0x01 {
                        nal_end = j;
                        break;
                    } else if data[j + 2] == 0x01 {
                        nal_end = j;
                        break;
                    }
                }
                j += 1;
            }

            if nal_end > nal_start {
                units.push((nal_start, nal_end));
            }
            i = nal_start;
        } else {
            i += 1;
        }
    }

    units
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
            nal_buffer: Vec::new(),
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
        self.nal_buffer.clear(); // Reset NAL buffer
        let mut stdout = stdout;

        // 5. Mark as running
        *self.running.lock().unwrap() = true;

        // 6. Start thread that reads H264 bytes, extracts NAL units, and emits via Tauri events
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
            info!("Video stream thread started for NAL unit extraction");
            let mut read_buf = vec![0u8; 65536];
            let mut nal_buffer: Vec<u8> = Vec::with_capacity(1024 * 1024); // 1MB buffer
            let mut nal_units_emitted = 0u64;
            let mut total_bytes_read = 0u64;
            let mut last_log_time = std::time::Instant::now();

            while *running.lock().unwrap() {
                // Read from screenrecord stdout
                match stdout.read(&mut read_buf) {
                    Ok(0) => {
                        info!(
                            "Video stream EOF ({} NAL units, {} bytes read)",
                            nal_units_emitted, total_bytes_read
                        );
                        // Emit any remaining buffered data as final NAL unit
                        if !nal_buffer.is_empty() {
                            if let Err(e) = app_handle.emit(VIDEO_FRAME_EVENT, &nal_buffer) {
                                error!("Failed to emit final NAL unit: {}", e);
                            } else {
                                nal_units_emitted += 1;
                                info!("Emitted final NAL unit ({} bytes)", nal_buffer.len());
                            }
                        }
                        break;
                    }
                    Ok(n) => {
                        total_bytes_read += n as u64;

                        // Append new data to NAL buffer
                        nal_buffer.extend_from_slice(&read_buf[..n]);

                        // Find and extract complete NAL units
                        let nal_units = find_nal_units(&nal_buffer);

                        if nal_units.is_empty() {
                            // No complete NAL units yet, keep buffering
                            // If buffer is getting too large, something is wrong
                            if nal_buffer.len() > 5 * 1024 * 1024 {
                                warn!("NAL buffer exceeds 5MB, clearing. Data may be corrupted.");
                                nal_buffer.clear();
                            }
                            continue;
                        }

                        // Extract complete NAL units (all except the last incomplete one)
                        let last_nal_end = nal_units.last().unwrap().1;

                        for (start, end) in nal_units.iter().take(nal_units.len() - 1) {
                            let start_idx = *start;
                            let end_idx = *end;
                            let nal_data = &nal_buffer[start_idx..end_idx];
                            if !nal_data.is_empty() {
                                if let Err(e) = app_handle.emit(VIDEO_FRAME_EVENT, nal_data) {
                                    error!("Failed to emit NAL unit: {}", e);
                                    break;
                                }
                                nal_units_emitted += 1;

                                // Log first few NAL units and periodically
                                let elapsed = last_log_time.elapsed();
                                if nal_units_emitted <= 5 || elapsed.as_secs() >= 5 {
                                    let nal_type = nal_data.first().map(|b| b & 0x1F).unwrap_or(0);
                                    info!(
                                        "NAL unit {} emitted (type={}, {} bytes)",
                                        nal_units_emitted,
                                        nal_type,
                                        nal_data.len()
                                    );
                                    last_log_time = std::time::Instant::now();
                                }
                            }
                        }

                        // Keep incomplete NAL unit in buffer for next read
                        if last_nal_end < nal_buffer.len() {
                            let remaining = nal_buffer[last_nal_end..].to_vec();
                            nal_buffer.clear();
                            nal_buffer.extend(remaining);
                        } else {
                            nal_buffer.clear();
                        }
                    }
                    Err(e) => {
                        error!(
                            "Stream read error after {} NAL units: {}",
                            nal_units_emitted, e
                        );
                        break;
                    }
                }
            }

            info!(
                "Video stream ended: {} NAL units emitted, {} bytes total",
                nal_units_emitted,
                total_bytes_read / 1_048_576
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
