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
//! H.264 Access-Unit Grouping (Option A / Approach A3):
//! The raw H.264 stream from screenrecord is parsed at NAL unit boundaries and
//! then GROUPED into complete access units (frames). One access unit = one
//! Tauri event. The frontend uses the first NAL unit byte to classify the
//! chunk as `key` (IDR slice present, NAL type 5) or `delta`.
//!
//! Access-unit boundaries:
//!   - New SPS (NAL type 7)        ⇒ new access unit begins
//!   - New PPS (NAL type 8)        ⇒ new access unit begins
//!   - New IDR slice (NAL type 5)  ⇒ new access unit begins
//!   - End-of-stream               ⇒ flush current accumulation

use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tracing::{error, info, warn};

/// Event name used to emit video frames to the frontend.
pub const VIDEO_FRAME_EVENT: &str = "video-frame";

/// Serialized payload for one access unit (one Tauri event).
///
/// `bytes` is the complete Annex-B byte stream for the access unit:
///   - keyframe: SPS + PPS + SEI + IDR + slice NALs (each with its start code)
///   - delta   : slice NAL(s) (each with its start code)
///
/// `key` is true iff the access unit contains an IDR slice (NAL type 5).
/// `has_vcl` is true iff the access unit contains at least one VCL slice
/// (NAL types 1 or 5), which means the AU is actually decodable.
#[derive(Clone, serde::Serialize)]
pub struct VideoFrameEvent<'a> {
    pub bytes: &'a [u8],
    pub key: bool,
}

/// H.264 NAL unit types we care about.
const NAL_TYPE_SLICE: u8 = 1; // non-IDR slice (delta frame)
const NAL_TYPE_IDR: u8 = 5; // IDR slice (key frame)
const NAL_TYPE_SPS: u8 = 7; // Sequence Parameter Set
const NAL_TYPE_PPS: u8 = 8; // Picture Parameter Set
const NAL_TYPE_AUD: u8 = 9; // Access Unit Delimiter (Android's screenrecord emits these between frames)

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

/// Find all NAL unit start positions in the buffer.
/// Returns vector of (start_pos, end_pos) tuples.
/// Find all NAL units in the buffer INCLUSIVE of their start code so callers
/// can append them to an access-unit payload as the decoder expects them.
///
/// Returns a vector of `(start_code_pos, end_pos)` for each NAL unit where:
///   * `start_code_pos` points at the FIRST byte of the start code
///     (i.e. `0x00` of `00 00 00 01` or `00 00 01`).
///   * `end_pos` points at the FIRST byte of the next start code, or at
///     `data.len()` if this is the last NAL unit in the buffer.
///
/// So the slice `data[start_code_pos..end_pos]` is the complete NAL unit
/// including its start code.
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

            // Start code position is `i`; the first byte AFTER the start code
            // (the NAL header) is at `i + start_code_len`. The NAL unit ends
            // where the next start code begins, or at end of buffer.
            let sc_pos = i;
            let mut nal_end = data.len();

            let mut j = sc_pos + start_code_len + 1;
            while j < data.len().saturating_sub(3) {
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

            if nal_end > sc_pos {
                units.push((sc_pos, nal_end));
            }
            i = sc_pos + start_code_len;
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
            info!("Video stream thread started: grouping NALs into access units");
            let mut read_buf = vec![0u8; 65536];
            let mut stream_buf: Vec<u8> = Vec::with_capacity(2 * 1024 * 1024); // 2MB accumulate
                                                                               // Current access unit being built. Holds ALL NAL units (with their
                                                                               // start codes preserved) for one frame:
                                                                               //   keyframe: SPS + PPS + SEI + IDR slice
                                                                               //   delta   : slice NAL(s)
            let mut current_au: Vec<u8> = Vec::with_capacity(512 * 1024);
            // Per-AU flags:
            //   au_has_idr = at least one IDR slice has been added
            //   au_has_vcl = at least one VCL slice (type 1 or 5) has been added.
            // We only flush an AU when au_has_vcl is true (so SPS/PPS-only AUs
            // are not emitted as junk frames).
            let mut au_has_idr = false;
            let mut au_has_vcl = false;
            let mut au_count = 0u64;
            let mut total_bytes_read = 0u64;
            let mut last_log_time = std::time::Instant::now();

            // Skip past an H.264 start code (3 or 4 bytes) at the start of
            // `buf` and return the offset of the NAL header byte. Returns None
            // if no complete start code is present.
            fn skip_start_code(buf: &[u8]) -> Option<usize> {
                if buf.len() >= 4
                    && buf[0] == 0x00
                    && buf[1] == 0x00
                    && buf[2] == 0x00
                    && buf[3] == 0x01
                {
                    Some(4)
                } else if buf.len() >= 3 && buf[0] == 0x00 && buf[1] == 0x00 && buf[2] == 0x01 {
                    Some(3)
                } else {
                    None
                }
            }

            // NAL type is the low 5 bits of the NAL header byte.
            fn nal_type(nal_header_byte: u8) -> u8 {
                nal_header_byte & 0x1F
            }

            // Emit a completed AU as a structured event with bytes + key flag.
            // Only emit if there is at least one VCL slice (decodable frame).
            // SPS/PPS-only accumulations (no VCL) are dropped, never emitted.
            fn flush_au(
                au: &mut Vec<u8>,
                has_idr: &mut bool,
                has_vcl: &mut bool,
                app: &AppHandle,
                count: &mut u64,
                last_log: &mut std::time::Instant,
            ) {
                if au.is_empty() || !*has_vcl {
                    au.clear();
                    *has_idr = false;
                    *has_vcl = false;
                    return;
                }
                let bytes = au.len();
                let keyframe = *has_idr;
                let payload = VideoFrameEvent {
                    bytes: au.as_slice(),
                    key: keyframe,
                };
                if let Err(e) = app.emit(VIDEO_FRAME_EVENT, payload) {
                    error!("Failed to emit access unit ({} bytes): {}", bytes, e);
                } else {
                    *count += 1;
                    if *count <= 3 || last_log.elapsed().as_secs() >= 5 {
                        info!(
                            "AU {} emitted: {} bytes, has_idr={}, has_vcl=true",
                            count, bytes, keyframe
                        );
                        *last_log = std::time::Instant::now();
                    }
                }
                au.clear();
                *has_idr = false;
                *has_vcl = false;
            }

            // Process whatever is currently in `stream_buf` as if it were the
            // last read. Used by the normal read loop AND by the EOF path so
            // any trailing bytes that formed complete NAL units get folded
            // into the final AU before we flush.
            //
            // Returns whether the stream buffer is now "safe to drain" — i.e.
            // the caller has accumulated all bytes that could ever arrive.
            fn fold_buf_to_au(
                stream_buf: &mut Vec<u8>,
                current_au: &mut Vec<u8>,
                au_has_idr: &mut bool,
                au_has_vcl: &mut bool,
                running: &Arc<Mutex<bool>>,
                app: &AppHandle,
                count: &mut u64,
                last_log: &mut std::time::Instant,
            ) {
                let nal_units = find_nal_units(stream_buf);
                if nal_units.is_empty() {
                    return;
                }

                // During normal reads we keep the LAST NAL buffered. At EOF,
                // however, there will be no more reads, so we instead process
                // the trailing NAL too — it represents the tail of the final AU.
                let processable = if *running.lock().unwrap() {
                    nal_units.len().saturating_sub(1)
                } else {
                    nal_units.len()
                };
                if processable == 0 {
                    return;
                }

                let mut consumed_to: usize = 0;
                for (sc_pos, end) in nal_units.iter().take(processable) {
                    consumed_to = *end;
                    if sc_pos >= end {
                        continue;
                    }
                    let header_offset_in_nal = match skip_start_code(&stream_buf[*sc_pos..*end]) {
                        Some(off) => off,
                        None => continue,
                    };
                    let header_pos = sc_pos + header_offset_in_nal;
                    if header_pos >= *end {
                        continue;
                    }
                    let nt = nal_type(stream_buf[header_pos]);

                    if nt == NAL_TYPE_AUD {
                        flush_au(current_au, au_has_idr, au_has_vcl, app, count, last_log);
                        continue;
                    }

                    current_au.extend_from_slice(&stream_buf[*sc_pos..*end]);

                    match nt {
                        NAL_TYPE_SLICE => *au_has_vcl = true,
                        NAL_TYPE_IDR => {
                            *au_has_idr = true;
                            *au_has_vcl = true;
                        }
                        _ => {}
                    }
                }

                if consumed_to > 0 {
                    stream_buf.drain(..consumed_to);
                }
            }

            while *running.lock().unwrap() {
                // Read from screenrecord stdout
                match stdout.read(&mut read_buf) {
                    Ok(0) => {
                        info!(
                            "Video stream EOF ({} access units emitted, {} bytes read)",
                            au_count, total_bytes_read
                        );
                        // FIX 2: any remaining bytes in stream_buf may form the
                        // tail of the last frame. Process them by clearing
                        // running = false, then fold_buf_to_au will pick up
                        // the trailing NALs because we no longer keep the last
                        // one buffered when shutting down.
                        *running.lock().unwrap() = false;
                        fold_buf_to_au(
                            &mut stream_buf,
                            &mut current_au,
                            &mut au_has_idr,
                            &mut au_has_vcl,
                            &running,
                            &app_handle,
                            &mut au_count,
                            &mut last_log_time,
                        );
                        // If the very last NAL still sits in stream_buf (no
                        // trailing start code ever arrived), it can't be
                        // classified as a complete unit — keep it in current_au
                        // so the final flush_au below can decide what to do.
                        if !stream_buf.is_empty() {
                            warn!(
                                "EOF: {} residual bytes in stream_buf, folding into last AU",
                                stream_buf.len()
                            );
                            current_au.extend_from_slice(stream_buf.as_slice());
                            stream_buf.clear();
                        }
                        flush_au(
                            &mut current_au,
                            &mut au_has_idr,
                            &mut au_has_vcl,
                            &app_handle,
                            &mut au_count,
                            &mut last_log_time,
                        );
                        break;
                    }
                    Ok(n) => {
                        total_bytes_read += n as u64;

                        // Append new data to stream buffer
                        stream_buf.extend_from_slice(&read_buf[..n]);

                        if stream_buf.len() > 10 * 1024 * 1024 {
                            warn!("Stream buffer exceeds 10MB, clearing. Stream may be corrupted.");
                            stream_buf.clear();
                            current_au.clear();
                            au_has_idr = false;
                            au_has_vcl = false;
                            continue;
                        }

                        fold_buf_to_au(
                            &mut stream_buf,
                            &mut current_au,
                            &mut au_has_idr,
                            &mut au_has_vcl,
                            &running,
                            &app_handle,
                            &mut au_count,
                            &mut last_log_time,
                        );
                    }
                    Err(e) => {
                        error!("Stream read error after {} access units: {}", au_count, e);
                        break;
                    }
                }
            }

            info!(
                "Video stream ended: {} access units emitted, {} bytes total read",
                au_count,
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
