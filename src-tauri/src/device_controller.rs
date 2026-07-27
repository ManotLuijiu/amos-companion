use crate::adb::{find_adb, run_adb};
use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub serial: String,
    pub model: String,
    pub status: String,
    pub resolution: Option<String>,
    pub battery: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceList {
    pub devices: Vec<DeviceInfo>,
}

/// Parse output of `adb devices -l` into DeviceInfo list.
fn parse_adb_devices(output: &str) -> Vec<DeviceInfo> {
    output
        .lines()
        .skip(1) // Skip "List of devices attached"
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                return None;
            }
            let serial = parts[0].to_string();
            let status = parts[1].to_string();

            // Parse -l output for model
            let model = parts
                .iter()
                .skip(2)
                .find(|s| s.starts_with("model:"))
                .map(|s| s.trim_start_matches("model:").to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            Some(DeviceInfo {
                serial,
                model,
                status,
                resolution: None,
                battery: None,
            })
        })
        .collect()
}

/// Get list of connected devices.
pub fn list_devices() -> Result<DeviceList, String> {
    let output = run_adb(&["devices", "-l"])?;
    let devices = parse_adb_devices(&output);
    Ok(DeviceList { devices })
}

/// Capture screenshot from a device and return as base64 PNG.
pub fn capture_screenshot(serial: &str) -> Result<String, String> {
    info!("Capturing screenshot from {}", serial);

    // Create temp file for screenshot
    let temp_path = std::env::temp_dir().join("amos_screenshot.png");
    let adb_path = find_adb();

    // Try exec-out first (faster, no temp file)
    let output = Command::new(&adb_path)
        .args(["-s", serial, "exec-out", "screencap", "-p"])
        .output();

    let png_data = match output {
        Ok(out) if out.status.success() && !out.stdout.is_empty() => {
            info!("Screenshot via exec-out: {} bytes", out.stdout.len());
            out.stdout
        }
        Ok(out) => {
            tracing::error!("Screenshot exec-out failed: status={}, stdout_len={}, stderr={}",
                out.status, out.stdout.len(), String::from_utf8_lossy(&out.stderr));
            // Fallback: use temp file method
            let _ = std::fs::remove_file(&temp_path);

            let capture_result = Command::new(&adb_path)
                .args(["-s", serial, "shell", "screencap", "-p", "/sdcard/amos_screen.png"])
                .output();

            match capture_result {
                Ok(cr) if cr.status.success() => {
                    let pull_result = Command::new(&adb_path)
                        .args(["-s", serial, "pull", "/sdcard/amos_screen.png", temp_path.to_str().unwrap_or("/tmp/amos_screen.png")])
                        .output();

                    match pull_result {
                        Ok(pr) if pr.status.success() => {
                            let data = std::fs::read(&temp_path).unwrap_or_default();
                            let _ = Command::new(&adb_path)
                                .args(["-s", serial, "shell", "rm", "/sdcard/amos_screen.png"])
                                .output();
                            let _ = std::fs::remove_file(&temp_path);
                            
                            if data.is_empty() {
                                return Err("Screenshot data is empty".to_string());
                            }
                            info!("Screenshot via pull: {} bytes", data.len());
                            data
                        }
                        Ok(pr) => {
                            return Err(format!("Failed to pull screenshot: {}", String::from_utf8_lossy(&pr.stderr)));
                        }
                        Err(e) => {
                            return Err(format!("Failed to pull screenshot: {}", e));
                        }
                    }
                }
                Ok(cr) => {
                    return Err(format!("Failed to capture screenshot: {}", String::from_utf8_lossy(&cr.stderr)));
                }
                Err(e) => {
                    return Err(format!("Failed to capture screenshot: {}", e));
                }
            }
        }
        Err(e) => {
            tracing::error!("Screenshot exec-out error: {}", e);
            // Fallback: use temp file method
            info!("Trying fallback screenshot method...");

            // Clean up any existing temp file
            let _ = std::fs::remove_file(&temp_path);

            // Capture to device storage
            let capture_result = run_adb(&[
                "-s",
                serial,
                "shell",
                "screencap",
                "-p",
                "/sdcard/amos_screen.png",
            ]);

            match capture_result {
                Ok(_) => {
                    // Pull the file
                    let pull_result = run_adb(&[
                        "-s",
                        serial,
                        "pull",
                        "/sdcard/amos_screen.png",
                        temp_path.to_str().unwrap_or("/tmp/amos_screen.png"),
                    ]);

                    match pull_result {
                        Ok(_) => {
                            let data = std::fs::read(&temp_path).unwrap_or_default();
                            // Clean up
                            let _ =
                                run_adb(&["-s", serial, "shell", "rm", "/sdcard/amos_screen.png"]);
                            let _ = std::fs::remove_file(&temp_path);

                            if data.is_empty() {
                                return Err("Screenshot data is empty".to_string());
                            }
                            info!("Screenshot via pull: {} bytes", data.len());
                            data
                        }
                        Err(e) => {
                            return Err(format!("Failed to pull screenshot: {}", e));
                        }
                    }
                }
                Err(e) => {
                    return Err(format!("Failed to capture screenshot: {}", e));
                }
            }
        }
    };

    // Validate PNG header
    if png_data.len() < 8 || &png_data[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(format!("Invalid PNG data: {} bytes", png_data.len()));
    }

    // Convert to base64
    let base64_str = general_purpose::STANDARD.encode(&png_data);

    info!(
        "Screenshot encoded, base64 size: {} chars",
        base64_str.len()
    );
    Ok(base64_str)
}

/// Get device info (resolution, battery, etc.)
pub fn get_device_info(serial: &str) -> Result<DeviceInfo, String> {
    // Get resolution
    let resolution_output = run_adb(&["-s", serial, "shell", "wm", "size"])?;
    let resolution = resolution_output
        .lines()
        .next()
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string());

    // Get battery
    let battery_output = run_adb(&[
        "-s", serial, "shell", "dumpsys", "battery", "|", "grep", "level",
    ])?;
    let battery = battery_output
        .lines()
        .next()
        .and_then(|l| l.split(':').nth(1))
        .and_then(|s| s.trim().parse::<i32>().ok());

    // Get model
    let model_output = run_adb(&["-s", serial, "shell", "getprop", "ro.product.model"])?;
    let model = model_output.trim().to_string();

    Ok(DeviceInfo {
        serial: serial.to_string(),
        model,
        status: "device".to_string(),
        resolution,
        battery,
    })
}

/// Send a tap event to the device.
pub fn tap(serial: &str, x: i32, y: i32) -> Result<(), String> {
    info!("Tap at {}x{} on {}", x, y, serial);
    run_adb(&[
        "-s",
        serial,
        "shell",
        "input",
        "tap",
        &x.to_string(),
        &y.to_string(),
    ])?;
    Ok(())
}

/// Send a swipe event to the device.
pub fn swipe(
    serial: &str,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    duration_ms: i32,
) -> Result<(), String> {
    info!("Swipe from {}x{} to {}x{} on {}", x1, y1, x2, y2, serial);
    run_adb(&[
        "-s",
        serial,
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

/// Send text input to the device.
pub fn text(serial: &str, text: &str) -> Result<(), String> {
    info!("Sending text to {}", serial);
    // Escape special characters for shell
    let escaped = text.replace(" ", "%s").replace("'", "\\'");
    run_adb(&["-s", serial, "shell", "input", "text", &escaped])?;
    Ok(())
}

/// Send key event to the device.
pub fn keyevent(serial: &str, keycode: &str) -> Result<(), String> {
    info!("Keyevent {} on {}", keycode, serial);
    run_adb(&["-s", serial, "shell", "input", "keyevent", keycode])?;
    Ok(())
}

/// Press back button.
pub fn back(serial: &str) -> Result<(), String> {
    keyevent(serial, "KEYCODE_BACK")
}

/// Press home button.
pub fn home(serial: &str) -> Result<(), String> {
    keyevent(serial, "KEYCODE_HOME")
}

/// Press enter/return key.
pub fn enter(serial: &str) -> Result<(), String> {
    keyevent(serial, "KEYCODE_ENTER")
}
