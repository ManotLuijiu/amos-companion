mod adb;
mod agent_manager;
mod config_store;
mod device_agent_installer;
mod device_controller;
mod scrcpy;
mod workspace_manager;

use agent_manager::AgentManager;
use config_store::ConfigStore;
use device_agent_installer as installer;
use device_controller::{DeviceInfo, DeviceList};
use scrcpy::{create_scrcpy_manager, is_scrcpy_available, ScrcpyManager};
use workspace_manager as wm;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tracing::{error, info};

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAgentStatus {
    pub installed: bool,
    pub path: String,
    pub os: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub agent_online: bool,
    pub agent_running: bool,
    #[serde(rename = "connected_devices")]
    pub connected_devices: Vec<String>,
    pub platform: String,
    #[serde(rename = "companion_version")]
    pub companion_version: String,
    #[serde(rename = "adb_version")]
    pub adb_version: String,
    #[serde(rename = "api_url")]
    pub api_url: String,
    #[serde(rename = "agent_pid")]
    pub agent_pid: Option<u32>,
    #[serde(rename = "error_message")]
    pub error_message: Option<String>,
}

impl Default for AgentStatus {
    fn default() -> Self {
        Self {
            agent_online: true,
            agent_running: false,
            connected_devices: vec![],
            platform: std::env::consts::OS.to_string(),
            companion_version: env!("CARGO_PKG_VERSION").to_string(),
            adb_version: String::new(),
            api_url: String::new(),
            agent_pid: None,
            error_message: None,
        }
    }
}

// ─── State ───────────────────────────────────────────────────────────────────

pub struct AppState {
    pub agent_manager: Arc<Mutex<AgentManager>>,
    pub config_store: Arc<Mutex<ConfigStore>>,
    pub scrcpy_manager: ScrcpyManager,
}

// ─── Tauri Commands ───────────────────────────────────────────────────────────

#[tauri::command]
async fn get_status(state: tauri::State<'_, AppState>) -> Result<AgentStatus, String> {
    let mut agent = state.agent_manager.lock().await;
    Ok(agent.get_status())
}

#[tauri::command]
async fn install_device_agent() -> Result<String, String> {
    info!("Installing device agent...");
    installer::install_or_update()?;
    Ok(format!(
        "Device agent installed at {:?}",
        installer::get_device_agent_dir()
    ))
}

#[tauri::command]
async fn get_device_agent_status() -> Result<DeviceAgentStatus, String> {
    Ok(DeviceAgentStatus {
        installed: installer::is_installed(),
        path: installer::get_device_agent_dir().to_string_lossy().to_string(),
        os: installer::get_os_info(),
    })
}

#[tauri::command]
async fn start_agent(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config_store.lock().await;
    let api_url = config.get_api_url();
    
    // Auto-install device-agent if not present
    if !installer::is_installed() {
        info!("Device agent not found, installing...");
        drop(config); // Release lock for git clone
        installer::install_or_update()?;
        config = state.config_store.lock().await;
    }

    let mut agent = state.agent_manager.lock().await;

    if agent.is_running() {
        return Err("Agent is already running".to_string());
    }

    // ─── Auto-setup: Get or create workspace ─────────────────────────────────
    
    // Check if we have workspace_id
    let workspace_id = config.get_workspace_id();
    
    if workspace_id.is_none() {
        info!("No workspace found, creating default workspace...");
        drop(config); // Release lock for HTTP call
        
        match wm::ensure_workspace_exists(&api_url).await {
            Ok(ws_id) => {
                config = state.config_store.lock().await;
                config.set_workspace_id(Some(ws_id.clone()));
                config.save().map_err(|e| format!("Failed to save workspace_id: {}", e))?;
            }
            Err(e) => {
                error!("Failed to create workspace: {}", e);
                return Err(format!("Failed to create workspace: {}", e));
            }
        }
    }

    // ─── Auto-setup: Register device-agent if needed ─────────────────────────
    
    let device_key = config.get_device_agent_key();
    let device_secret = config.get_device_agent_secret();
    let saved_agent_id = config.get_agent_id();
    
    if device_key.is_none() || device_secret.is_none() {
        info!("No device-agent credentials found, registering...");
        
        // Get workspace_id before dropping config
        let ws_id = config.get_workspace_id().unwrap_or_default();
        drop(config); // Release lock for HTTP call
        
        let hostname = wm::get_hostname();
        
        match wm::register_device_agent(&api_url, &ws_id, &hostname).await {
            Ok((api_key, api_secret, agent_id)) => {
                config = state.config_store.lock().await;
                // Clone before saving since we'll use them again
                config.set_device_agent_key(Some(api_key.clone()));
                config.set_device_agent_secret(Some(api_secret.clone()));
                config.save().map_err(|e| format!("Failed to save credentials: {}", e))?;
                
                info!("Device-agent registered successfully");
                
                // Now start the agent with credentials
                info!("Starting AMOS device agent with API URL: {}", api_url);
                
                match agent.start(&api_url, &agent_id, Some(api_key), Some(api_secret), Some(ws_id)).await {
                    Ok(_) => {
                        info!("Agent started successfully");
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        let status = agent.get_status();
                        app.emit("status-update", status)
                            .map_err(|e| e.to_string())?;
                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to start agent: {}", e);
                        let status = agent.get_status();
                        let _ = app.emit("status-update", status);
                        Err(e.to_string())
                    }
                }
            }
            Err(e) => {
                error!("Failed to register device-agent: {}", e);
                Err(format!("Failed to register device-agent: {}", e))
            }
        }
    } else {
        // We have credentials, just start the agent
        let ws_id = config.get_workspace_id().unwrap_or_default();
        
        info!("Starting AMOS device agent with API URL: {}", api_url);
        
        match agent.start(&api_url, &saved_agent_id, device_key, device_secret, Some(ws_id)).await {
            Ok(_) => {
                info!("Agent started successfully");
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                let status = agent.get_status();
                app.emit("status-update", status)
                    .map_err(|e| e.to_string())?;
                Ok(())
            }
            Err(e) => {
                error!("Failed to start agent: {}", e);
                let status = agent.get_status();
                let _ = app.emit("status-update", status);
                Err(e.to_string())
            }
        }
    }
}

#[tauri::command]
async fn stop_agent(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut agent = state.agent_manager.lock().await;

    if !agent.is_running() {
        return Err("Agent is not running".to_string());
    }

    info!("Stopping AMOS device agent");
    agent.stop().await;

    let status = agent.get_status();
    app.emit("status-update", status)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn save_config(
    api_url: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config_store.lock().await;
    config.set_api_url(api_url);
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn open_web_ui() -> Result<(), String> {
    open::that("https://app.amos.moo-vpn.online/devices")
        .map_err(|e| e.to_string())
}

// ─── Device Control Commands ─────────────────────────────────────────────────

#[tauri::command]
async fn get_devices() -> Result<DeviceList, String> {
    device_controller::list_devices()
}

#[tauri::command]
async fn get_device_info(serial: String) -> Result<DeviceInfo, String> {
    device_controller::get_device_info(&serial)
}

#[tauri::command]
async fn capture_screenshot(serial: String) -> Result<String, String> {
    device_controller::capture_screenshot(&serial)
}

#[tauri::command]
async fn device_tap(serial: String, x: i32, y: i32) -> Result<(), String> {
    device_controller::tap(&serial, x, y)
}

#[tauri::command]
async fn device_swipe(
    serial: String,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    duration_ms: i32,
) -> Result<(), String> {
    device_controller::swipe(&serial, x1, y1, x2, y2, duration_ms)
}

#[tauri::command]
async fn device_text(serial: String, text: String) -> Result<(), String> {
    device_controller::text(&serial, &text)
}

#[tauri::command]
async fn device_back(serial: String) -> Result<(), String> {
    device_controller::back(&serial)
}

#[tauri::command]
async fn device_home(serial: String) -> Result<(), String> {
    device_controller::home(&serial)
}

#[tauri::command]
async fn device_enter(serial: String) -> Result<(), String> {
    device_controller::enter(&serial)
}

// ─── Scrcpy Commands ──────────────────────────────────────────────────────────

#[tauri::command]
async fn is_scrcpy_available_cmd() -> Result<bool, String> {
    Ok(is_scrcpy_available())
}

#[tauri::command]
async fn start_scrcpy(
    serial: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut manager = state.scrcpy_manager.lock().await;
    
    // Update the serial if different
    let new_stream = scrcpy::ScrcpyStream::new(serial.clone());
    *manager = new_stream;
    
    manager.start()
}

#[tauri::command]
async fn stop_scrcpy(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut manager = state.scrcpy_manager.lock().await;
    manager.stop();
    Ok(())
}

// ─── Main ─────────────────────────────────────────────────────────────────────

pub fn run() {
    // Initialize logging
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("amos-companion")
        .join("logs");

    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::daily(&log_dir, "amos-companion.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("AMOS Companion v{} starting", env!("CARGO_PKG_VERSION"));

    // Initialize config
    let config_store = ConfigStore::load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load config: {}, using defaults", e);
        ConfigStore::default()
    });

    // Initialize agent manager
    let agent_manager = AgentManager::new();

    let app_state = AppState {
        agent_manager: Arc::new(Mutex::new(agent_manager)),
        config_store: Arc::new(Mutex::new(config_store)),
        scrcpy_manager: create_scrcpy_manager(String::new()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_status,
            start_agent,
            stop_agent,
            save_config,
            open_web_ui,
            install_device_agent,
            get_device_agent_status,
            get_devices,
            get_device_info,
            capture_screenshot,
            start_scrcpy,
            stop_scrcpy,
            is_scrcpy_available_cmd,
            device_tap,
            device_swipe,
            device_text,
            device_back,
            device_home,
            device_enter,
        ])
        .setup(|_app| {
            info!("Tauri app setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
