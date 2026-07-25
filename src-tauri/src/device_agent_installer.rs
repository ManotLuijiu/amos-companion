use std::path::PathBuf;
use std::process::Command;
use tracing::{error, info, warn};

/// Device agent installer - clones and manages the device-agent from GitHub
const DEVICE_AGENT_REPO: &str = "https://github.com/ManotLuijiu/amos-device-agent.git";

/// Get the install directory for device-agent based on OS
pub fn get_device_agent_dir() -> PathBuf {
    let base = match dirs::data_dir() {
        Some(dir) => dir,
        None => {
            // Fallback to home directory
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
        }
    };

    base.join("amos-companion").join("device-agent")
}

/// Check if device-agent is already installed
pub fn is_installed() -> bool {
    let dir = get_device_agent_dir();
    dir.join("amos_device_agent").exists()
}

/// Get the working directory for running device-agent commands
pub fn get_working_dir() -> PathBuf {
    get_device_agent_dir()
}

/// Clone or update the device-agent repository
pub fn install_or_update() -> Result<(), String> {
    let dir = get_device_agent_dir();

    if dir.exists() {
        // Update existing installation
        info!("Device agent found at {:?}, updating...", dir);
        update_existing(&dir)
    } else {
        // Fresh install
        info!("Device agent not found, cloning from {}", DEVICE_AGENT_REPO);
        clone_fresh(&dir)
    }
}

fn clone_fresh(dir: &PathBuf) -> Result<(), String> {
    // Create parent directories
    if let Some(parent) = dir.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    info!("Cloning device-agent to {:?}", dir);

    let output = Command::new("git")
        .args(["clone", DEVICE_AGENT_REPO, dir.to_str().unwrap_or("")])
        .output()
        .map_err(|e| format!("Failed to run git clone: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Git clone failed: {}", stderr);
        return Err(format!("Failed to clone device-agent: {}", stderr));
    }

    info!("Device-agent cloned successfully");
    install_dependencies(dir)
}

fn update_existing(dir: &PathBuf) -> Result<(), String> {
    info!("Pulling latest device-agent...");

    let output = Command::new("git")
        .args(["pull", "origin", "main"])
        .current_dir(dir)
        .output()
        .map_err(|e| format!("Failed to run git pull: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("Git pull failed (may be on different branch): {}", stderr);
        // Not a fatal error - existing installation still works
    } else {
        info!("Device-agent updated successfully");
    }

    install_dependencies(dir)
}

fn install_dependencies(dir: &PathBuf) -> Result<(), String> {
    info!("Installing device-agent dependencies...");

    // Try uv first (preferred)
    let uv_result = try_uv_install(dir);

    if uv_result.is_ok() {
        info!("Dependencies installed via uv");
        return Ok(());
    }

    // Fallback to pip
    let pip_result = try_pip_install(dir);

    if pip_result.is_ok() {
        info!("Dependencies installed via pip");
        return Ok(());
    }

    Err("Failed to install dependencies. Please install uv or pip.".to_string())
}

fn try_uv_install(dir: &PathBuf) -> Result<(), String> {
    // Check if uv is available
    let uv_check = Command::new("sh").arg("-c").arg("which uv").output();

    match uv_check {
        Ok(out) if out.status.success() => {
            info!("uv found, installing...");

            let output = Command::new("uv")
                .args(["pip", "install", "-e", "."])
                .current_dir(dir)
                .output()
                .map_err(|e| format!("uv pip install failed: {}", e))?;

            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("uv pip install failed: {}", stderr))
            }
        }
        _ => Err("uv not found".to_string()),
    }
}

fn try_pip_install(dir: &PathBuf) -> Result<(), String> {
    info!("Trying pip install...");

    // Try pip3 first, then pip
    let pip_cmd = if Command::new("sh")
        .arg("-c")
        .arg("which pip3")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        "pip3"
    } else {
        "pip"
    };

    let output = Command::new(pip_cmd)
        .args(["install", "-e", "."])
        .current_dir(dir)
        .output()
        .map_err(|e| format!("pip install failed: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("pip install failed: {}", stderr))
    }
}

/// Get OS-specific information for logging
pub fn get_os_info() -> String {
    format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
}
