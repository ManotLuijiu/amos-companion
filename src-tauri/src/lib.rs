mod adb;
mod agent_manager;
mod config_store;
mod dependency_manager;
mod device_agent_installer;
mod device_controller;
mod install_mode;
mod scrcpy;
mod scrcpy_server;
mod video_stream;
mod workspace_manager;
mod ws_scrcpy_server;

use agent_manager::AgentManager;
use config_store::ConfigStore;
use dependency_manager as deps;
use deps::DependencyStatus;
use device_agent_installer as installer;
use device_controller::{DeviceInfo, DeviceList};
use workspace_manager::RegisteredDevice;
use scrcpy::{create_scrcpy_manager, is_scrcpy_available, ScrcpyManager};
use scrcpy_server::{create_scrcpy_server, ScrcpyServerManager};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tokio::sync::Mutex;
use tracing::{error, info};
use video_stream::VideoStream;
use workspace_manager as wm;
use ws_scrcpy_server::{create_ws_scrcpy_server, WsScrcpyServerManager, WsScrcpyStatus};

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
    pub scrcpy_server: ScrcpyServerManager,
    pub video_stream: Arc<Mutex<Option<VideoStream>>>,
    pub ws_scrcpy_server: WsScrcpyServerManager,
}

// ─── Tauri Commands ───────────────────────────────────────────────────────────

#[tauri::command]
async fn get_status(state: tauri::State<'_, AppState>) -> Result<AgentStatus, String> {
    let mut agent = state.agent_manager.lock().await;
    Ok(agent.get_status())
}

#[tauri::command]
async fn get_agent_logs(state: tauri::State<'_, AppState>) -> Result<(String, bool), String> {
    let mut agent = state.agent_manager.lock().await;

    // Get the log directory path
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("amos-companion")
        .join("logs");
    let stderr_file = log_dir.join("agent-stderr.log");

    let is_alive = agent.is_running();

    // Read last 50 lines of stderr
    let tail = if stderr_file.exists() {
        match std::fs::read_to_string(&stderr_file) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().rev().take(50).collect();
                lines.into_iter().rev().collect::<Vec<_>>().join("\n")
            }
            Err(e) => format!("Failed to read log: {}", e),
        }
    } else {
        "No agent log file found".to_string()
    };

    Ok((tail, is_alive))
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
async fn install_adb() -> Result<String, String> {
    info!("Installing ADB...");
    deps::install_adb().await.map_err(|e| e.to_string())?;
    let adb_path = adb::find_adb();
    Ok(format!("ADB installed at: {}", adb_path))
}

#[tauri::command]
async fn get_device_agent_status() -> Result<DeviceAgentStatus, String> {
    Ok(DeviceAgentStatus {
        installed: installer::is_installed(),
        path: installer::get_device_agent_dir()
            .to_string_lossy()
            .to_string(),
        os: installer::get_os_info(),
    })
}

// ─── Mirror Dependency Management ─────────────────────────────────────────────

#[tauri::command]
async fn install_mirror_deps() -> Result<String, String> {
    deps::install_all().await
}

#[tauri::command]
fn get_mirror_deps_status() -> DependencyStatus {
    DependencyStatus::check()
}

#[tauri::command]
async fn start_ws_scrcpy_server(
    state: tauri::State<'_, AppState>,
    port: Option<u16>,
) -> Result<String, String> {
    info!("Starting ws-scrcpy server...");
    let mut server = state.ws_scrcpy_server.lock().await;
    
    if let Some(p) = port {
        server.set_port(p);
    }
    
    server.start().await
}

#[tauri::command]
async fn stop_ws_scrcpy_server(state: tauri::State<'_, AppState>) -> Result<(), String> {
    info!("Stopping ws-scrcpy server...");
    let mut server = state.ws_scrcpy_server.lock().await;
    server.stop();
    Ok(())
}

#[tauri::command]
async fn get_ws_scrcpy_status(state: tauri::State<'_, AppState>) -> Result<WsScrcpyStatus, String> {
    let server = state.ws_scrcpy_server.lock().await;
    Ok(server.get_status())
}

#[tauri::command]
async fn sign_in(
    state: tauri::State<'_, AppState>,
    api_url: String,
    email: String,
    password: String,
) -> Result<(), String> {
    info!("Signing in as {}", email);

    // Sign in via better-auth API
    let (user_id, user_email) = wm::sign_in(&api_url, &email, &password).await?;

    // Get or create default workspace
    match wm::ensure_workspace_exists(&api_url, &user_id).await {
        Ok(ws_id) => {
            let mut config = state.config_store.lock().await;
            config.set_api_url(api_url.clone());
            config.set_user_id(Some(user_id.clone()));
            config.set_user_email(Some(user_email.clone()));
            config.set_workspace_id(Some(ws_id.clone()));
            config
                .save()
                .map_err(|e| format!("Failed to save config: {}", e))?;
            info!("Sign in successful: {} ({}) with workspace {}", user_email, user_id, ws_id);
        }
        Err(e) => {
            // Save without workspace - workspace will be created later
            let mut config = state.config_store.lock().await;
            config.set_api_url(api_url.clone());
            config.set_user_id(Some(user_id.clone()));
            config.set_user_email(Some(user_email.clone()));
            config
                .save()
                .map_err(|e| format!("Failed to save config: {}", e))?;
            info!("Sign in successful: {} ({}), workspace error: {}", user_email, user_id, e);
        }
    }

    Ok(())
}

#[tauri::command]
async fn get_user_info(
    state: tauri::State<'_, AppState>,
) -> Result<Option<(String, String)>, String> {
    let config = state.config_store.lock().await;
    let user_id = config.get_user_id();
    let user_email = config.get_user_email();

    match (user_id, user_email) {
        (Some(id), Some(email)) => Ok(Some((id, email))),
        _ => Ok(None),
    }
}

#[tauri::command]
async fn get_user_info_full(
    state: tauri::State<'_, AppState>,
) -> Result<Option<(String, String, String)>, String> {
    let config = state.config_store.lock().await;
    let user_id = config.get_user_id();
    let user_email = config.get_user_email();
    let workspace_id = config.get_workspace_id();

    match (user_id, user_email, workspace_id) {
        (Some(id), Some(email), Some(ws_id)) => Ok(Some((id, email, ws_id))),
        _ => Ok(None),
    }
}

#[tauri::command]
async fn sign_in_manual(
    state: tauri::State<'_, AppState>,
    api_url: String,
    user_id: String,
) -> Result<(), String> {
    info!("Manual sign-in for user {}", user_id);

    // Save to config without email verification
    let mut config = state.config_store.lock().await;
    config.set_api_url(api_url);
    config.set_user_id(Some(user_id));
    config.set_user_email(Some("OAuth User".to_string()));
    config
        .save()
        .map_err(|e| format!("Failed to save config: {}", e))?;

    info!("Manual sign-in successful");
    Ok(())
}

#[tauri::command]
async fn sign_out(state: tauri::State<'_, AppState>) -> Result<(), String> {
    info!("Signing out...");

    // Clear user credentials from config
    let mut config = state.config_store.lock().await;
    config.set_user_id(None);
    config.set_user_email(None);
    config.set_workspace_id(None);
    config.set_device_agent_key(None);
    config.set_device_agent_secret(None);
    config
        .save()
        .map_err(|e| format!("Failed to save config: {}", e))?;

    info!("Sign-out successful");
    Ok(())
}

#[tauri::command]
async fn open_url(url: String) -> Result<(), String> {
    info!("Opening URL: {}", url);
    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(&url)
        .spawn()
        .map_err(|e| format!("Failed to open URL: {}", e))?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(&url)
        .spawn()
        .map_err(|e| format!("Failed to open URL: {}", e))?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(["/C", "start", "", &url])
        .spawn()
        .map_err(|e| format!("Failed to open URL: {}", e))?;

    Ok(())
}

#[tauri::command]
fn get_install_mode() -> install_mode::InstallMode {
    install_mode::detect_install_mode()
}

#[tauri::command]
async fn sign_in_oauth(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    api_url: String,
) -> Result<(), String> {
    use std::net::TcpListener;
    use std::thread;
    use std::time::Duration;

    info!("Starting OAuth flow for user login...");

    // Bind to port 0 to get an available port
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Failed to bind to localhost: {}", e))?;

    let addr = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?;
    let port = addr.port();
    info!("OAuth callback server listening on port {}", port);

    // Create a channel to receive the auth result
    let (tx, rx) = std::sync::mpsc::channel::<Result<(String, String), String>>();

    // Start a local HTTP server to receive the OAuth callback
    let server_tx = tx.clone();
    let _server = thread::spawn(move || {
        // Set a timeout so we don't hang forever
        listener.set_nonblocking(true).ok();

        // Accept connection with timeout - match OAuth 120s wait window
        let mut attempts = 0;
        while attempts < 1200 {
            // 120 seconds timeout (matches recv_timeout in caller)
            thread::sleep(Duration::from_millis(100));
            attempts += 1;

            if let Ok((mut stream, _)) = listener.accept() {
                use std::io::{Read, Write};
                let mut buffer = [0u8; 2048];
                if stream.read(&mut buffer).is_ok() {
                    let request = String::from_utf8_lossy(&buffer);
                    info!(
                        "Received callback request: {}",
                        &request[..request.len().min(200)]
                    );

                    // Parse the callback URL from the request
                    // Format: GET /callback?user_id=xxx&email=xxx HTTP/1.1
                    if let Some(query_start) = request.find("/callback?") {
                        let query = &request[query_start + 10..];
                        let params: Vec<&str> = query.split('&').collect();

                        let mut user_id = String::new();
                        let mut email = String::new();

                        for param in params {
                            if param.starts_with("user_id=") {
                                user_id = param[8..]
                                    .split_whitespace()
                                    .next()
                                    .unwrap_or("")
                                    .to_string();
                                user_id = percent_encoding::percent_decode_str(&user_id)
                                    .decode_utf8_lossy()
                                    .to_string();
                            } else if param.starts_with("email=") {
                                email = param[6..]
                                    .split_whitespace()
                                    .next()
                                    .unwrap_or("")
                                    .to_string();
                                email = percent_encoding::percent_decode_str(&email)
                                    .decode_utf8_lossy()
                                    .to_string();
                            }
                        }

                        // Send success response
                        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<!DOCTYPE html><html><head><title>AMOS Companion</title></head><body><h1>Login Successful!</h1><p>You can close this window and return to AMOS Companion.</p><script>window.close();</script></body></html>";
                        stream.write_all(response.as_bytes()).ok();

                        if !user_id.is_empty() {
                            info!("Got user_id: {}", user_id);
                            server_tx.send(Ok((user_id.clone(), email.clone()))).ok();
                        } else {
                            info!("No user_id in callback");
                            server_tx
                                .send(Err("No user_id in callback".to_string()))
                                .ok();
                        }
                    }
                }
                break;
            }
        }
    });

    // Get the Google OAuth URL from our backend
    // First, call our endpoint to get the proper Google OAuth URL
    // Convert API URL to frontend URL
    // https://amos-api.moo-vpn.online → https://app.amos.moo-vpn.online
    let frontend_url = api_url
        .replace("://api.", "://app.")
        .replace("://api/", "://app/")
        .replace("/api", "")
        .replace("amos-api.", "app.amos.");

    let callback_base = format!("http://127.0.0.1:{}", port);
    let google_url_endpoint = format!(
        "{}/api/auth/companion/google-url?callbackUrl={}",
        frontend_url, callback_base
    );

    info!("Getting Google OAuth URL from: {}", google_url_endpoint);

    // Fetch the Google OAuth URL from our backend
    let client = reqwest::Client::new();
    let oauth_response = client
        .get(&google_url_endpoint)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to get OAuth URL: {}", e))?;

    if !oauth_response.status().is_success() {
        let status = oauth_response.status();
        let error_text = oauth_response.text().await.unwrap_or_default();
        return Err(format!(
            "Failed to get OAuth URL: {} - {}",
            status, error_text
        ));
    }

    let oauth_data: serde_json::Value = oauth_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse OAuth response: {}", e))?;

    let oauth_url = oauth_data["url"]
        .as_str()
        .ok_or_else(|| "No URL in OAuth response".to_string())?;

    info!("Got Google OAuth URL, opening browser...");

    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(&oauth_url)
        .spawn()
        .map_err(|e| format!("Failed to open OAuth URL: {}", e))?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(oauth_url)
        .spawn()
        .map_err(|e| format!("Failed to open OAuth URL: {}", e))?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(["/C", "start", "", &oauth_url])
        .spawn()
        .map_err(|e| format!("Failed to open OAuth URL: {}", e))?;

    // Wait for the callback (with timeout)
    info!("Waiting for OAuth callback...");
    let result = rx.recv_timeout(Duration::from_secs(120));

    match result {
        Ok(Ok((user_id, email))) => {
            info!("OAuth successful: user_id={}, email={}", user_id, email);

            // Get or create default workspace
            let api_url_clone = api_url.clone();
            let user_id_clone = user_id.clone();

            // Save user info to config first
            {
                let mut config = state.config_store.lock().await;
                config.set_api_url(api_url.clone());
                config.set_user_id(Some(user_id.clone()));
                config.set_user_email(Some(email.clone()));
                config
                    .save()
                    .map_err(|e| format!("Failed to save config: {}", e))?;
            }

            // Fetch workspace from backend FIRST
            match wm::ensure_workspace_exists(&api_url_clone, &user_id_clone).await {
                Ok(ws_id) => {
                    let mut config = state.config_store.lock().await;
                    config.set_workspace_id(Some(ws_id.clone()));
                    config
                        .save()
                        .map_err(|e| format!("Failed to save workspace: {}", e))?;
                    info!("Workspace set: {}", ws_id);
                }
                Err(e) => {
                    info!("Workspace will be created on first agent start: {}", e);
                    // Continue anyway - workspace will be created on first agent start
                }
            }

            // Emit login success event to frontend AFTER config is complete
            app.emit(
                "login-success",
                serde_json::json!({
                    "user_id": user_id,
                    "email": email
                }),
            )
            .map_err(|e| format!("Failed to emit event: {}", e))?;

            Ok(())
        }
        Ok(Err(e)) => {
            error!("OAuth failed: {}", e);
            Err(e)
        }
        Err(_) => {
            error!("OAuth timeout");
            Err("OAuth login timed out. Please try again.".to_string())
        }
    }
}

#[tauri::command]
async fn start_agent(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut config = state.config_store.lock().await;
    let api_url = config.get_api_url();

    // Validate api_url is not empty
    if api_url.trim().is_empty() {
        error!("start_agent: api_url is empty — not signed in / configured");
        return Err("API URL not configured — sign in first".to_string());
    }

    // ─── Check if user is signed in ─────────────────────────────────────────────
    let user_id = config.get_user_id().ok_or_else(|| {
        error!("Not signed in - please sign in first");
        "Please sign in first using sign_in()".to_string()
    })?;

    // Auto-update device-agent to ensure latest code
    info!("Checking device-agent for updates...");
    drop(config); // Release lock for git operations
    if let Err(e) = installer::install_or_update() {
        error!("Device-agent update failed: {}", e);
        // Continue anyway - existing installation may still work
    } else {
        info!("Device-agent updated successfully");
    }
    config = state.config_store.lock().await;

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

        match wm::ensure_workspace_exists(&api_url, &user_id).await {
            Ok(ws_id) => {
                config = state.config_store.lock().await;
                config.set_workspace_id(Some(ws_id.clone()));
                config
                    .save()
                    .map_err(|e| format!("Failed to save workspace_id: {}", e))?;
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

        match wm::register_device_agent(&api_url, &ws_id, &user_id, &hostname).await {
            Ok((api_key, api_secret, agent_id)) => {
                config = state.config_store.lock().await;
                // Clone before saving since we'll use them again
                config.set_device_agent_key(Some(api_key.clone()));
                config.set_device_agent_secret(Some(api_secret.clone()));
                config
                    .save()
                    .map_err(|e| format!("Failed to save credentials: {}", e))?;

                info!("Device-agent registered successfully");

                // Now start the agent with credentials
                info!("Starting AMOS device agent with API URL: {}", api_url);

                match agent
                    .start(
                        &api_url,
                        &agent_id,
                        Some(api_key),
                        Some(api_secret),
                        Some(ws_id),
                        Some(user_id.clone()),
                    )
                    .await
                {
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

        match agent
            .start(
                &api_url,
                &saved_agent_id,
                device_key,
                device_secret,
                Some(ws_id),
                Some(user_id.clone()),
            )
            .await
        {
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
async fn stop_agent(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
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
async fn save_config(api_url: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut config = state.config_store.lock().await;
    config.set_api_url(api_url);
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn open_web_ui() -> Result<(), String> {
    open::that("https://app.amos.moo-vpn.online/devices").map_err(|e| e.to_string())
}

// ─── Device Control Commands ─────────────────────────────────────────────────

#[tauri::command]
async fn get_devices() -> Result<DeviceList, String> {
    device_controller::list_devices()
}

#[tauri::command]
async fn get_registered_devices(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<RegisteredDevice>, String> {
    let config = state.config_store.lock().await;
    let user_id = config.get_user_id().ok_or("Not logged in")?;
    let workspace_id = config.get_workspace_id().ok_or("No workspace")?;
    let api_url = config.get_api_url();
    info!("get_registered_devices: user_id={}, workspace_id={}, api_url={}", user_id, workspace_id, api_url);
    if api_url.is_empty() {
        return Err("API URL not configured".to_string());
    }
    drop(config);
    let result = workspace_manager::get_registered_devices(&api_url, &user_id, &workspace_id).await;
    match &result {
        Ok(devices) => info!("get_registered_devices returned {} devices", devices.len()),
        Err(e) => error!("get_registered_devices failed: {}", e),
    }
    result
}

#[tauri::command]
async fn update_device_name(
    deviceId: String,
    name: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let config = state.config_store.lock().await;
    let user_id = config.get_user_id().ok_or("Not logged in")?;
    let workspace_id = config.get_workspace_id().ok_or("No workspace")?;
    let api_url = config.get_api_url();
    info!("update_device_name: deviceId={}, name={}, user_id={}, workspace_id={}", deviceId, name, user_id, workspace_id);
    if api_url.is_empty() {
        return Err("API URL not configured".to_string());
    }
    drop(config);
    workspace_manager::update_device_name(&api_url, &user_id, &workspace_id, &deviceId, &name).await
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

#[tauri::command(rename_all = "snake_case")]
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
) -> Result<scrcpy::ScrcpyLaunchResult, String> {
    let mut manager = state.scrcpy_manager.lock().await;

    // Stop any existing scrcpy process before replacing the manager
    // to avoid orphaning std::process::Child handles.
    if manager.is_active() {
        info!("Stopping existing scrcpy before starting new one");
        manager.stop();
    }

    // Update the serial if different
    let new_stream = scrcpy::ScrcpyStream::new(serial.clone());
    *manager = new_stream;

    let result = manager.start().await;
    if result.success {
        Ok(result)
    } else {
        Err(result.message)
    }
}

#[tauri::command]
async fn stop_scrcpy(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut manager = state.scrcpy_manager.lock().await;
    manager.stop();
    Ok(())
}

// ─── Scrcpy-Server WebSocket Commands ─────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScrcpyStreamInfo {
    pub width: u32,
    pub height: u32,
    pub running: bool,
    pub event_name: String,
}

#[tauri::command]
async fn start_scrcpy_mirror(
    serial: String,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<ScrcpyStreamInfo, String> {
    info!("Starting scrcpy-server mirror for device: {}", serial);
    let mut server = state.scrcpy_server.lock().await;

    // Update serial
    *server = scrcpy_server::ScrcpyServer::new(serial.clone());

    // Start server with Tauri events (emits frames via scrcpy-frame event)
    // Returns (width, height, process_alive) where process_alive indicates
    // whether scrcpy-server.jar is actually running on the device.
    let (width, height, process_alive) = server.start_with_events(&app)?;

    // Emit debug info AFTER start_with_events returns (frontend has set up listener by now)
    let debug_info = server.get_debug_info();
    if !debug_info.is_empty() {
        let _ = app.emit("scrcpy-debug", debug_info);
    }

    if !process_alive {
        // Truthfully report failure to the frontend so it doesn't show
        // "Server started (1920x1920)" when the scrcpy-server process is not
        // actually alive. The frontend should fall back to ADB screenrecord
        // when running=false comes back.
        info!(
            "scrcpy-server process not alive on device after launch for {}",
            serial
        );
        let _ = app.emit(
            "scrcpy-stream-started",
            serde_json::json!({
                "serial": serial,
                "running": false,
                "reason": "scrcpy-server process not alive on device",
            }),
        );
        return Ok(ScrcpyStreamInfo {
            width: 0,
            height: 0,
            running: false,
            event_name: scrcpy_server::ScrcpyServer::SCRCPY_FRAME_EVENT.to_string(),
        });
    }

    info!(
        "scrcpy-server mirror started for {} ({}x{})",
        serial, width, height
    );

    Ok(ScrcpyStreamInfo {
        width,
        height,
        running: true,
        event_name: scrcpy_server::ScrcpyServer::SCRCPY_FRAME_EVENT.to_string(),
    })
}

#[tauri::command]
async fn stop_scrcpy_mirror(state: tauri::State<'_, AppState>) -> Result<(), String> {
    info!("Stopping scrcpy-server mirror");
    let mut server = state.scrcpy_server.lock().await;
    server.stop();
    Ok(())
}

// Legacy commands (kept for backward compatibility)
#[tauri::command]
async fn start_mirror(serial: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    info!("Starting mirror for device: {}", serial);
    let mut server = state.scrcpy_server.lock().await;

    // Update serial
    *server = scrcpy_server::ScrcpyServer::new(serial);

    // Start server (legacy - returns WebSocket URL)
    if server.start()? {
        Ok(format!("ws://127.0.0.1:{}", server.get_local_port()))
    } else {
        Err("scrcpy-server process not alive on device".to_string())
    }
}

#[tauri::command]
async fn stop_mirror(state: tauri::State<'_, AppState>) -> Result<(), String> {
    info!("Stopping mirror");
    let mut server = state.scrcpy_server.lock().await;
    server.stop();
    Ok(())
}

#[tauri::command]
async fn mirror_control(
    _serial: String,
    action: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let server = state.scrcpy_server.lock().await;

    match action.as_str() {
        "back" => server.back(),
        "home" => server.home(),
        "enter" => server.enter(),
        _ => Err(format!("Unknown action: {}", action)),
    }
}

#[tauri::command]
async fn mirror_tap(x: i32, y: i32, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let server = state.scrcpy_server.lock().await;
    server.tap(x, y)
}

#[tauri::command]
async fn mirror_swipe(
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    duration_ms: i32,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let server = state.scrcpy_server.lock().await;
    server.swipe(x1, y1, x2, y2, duration_ms)
}

// ─── Video Stream Commands ─────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct StreamInfo {
    pub width: u32,
    pub height: u32,
    pub running: bool,
    pub event_name: String,
}

#[tauri::command]
async fn start_video_stream(
    serial: String,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<StreamInfo, String> {
    info!("Starting video stream for device: {}", serial);

    // Stop any existing stream
    {
        let mut stream = state.video_stream.lock().await;
        if let Some(ref mut s) = *stream {
            s.stop();
        }
        *stream = None;
    }

    // Create and start new stream (uses Tauri events, not WebSocket)
    let mut video_stream = VideoStream::new(serial);
    let (width, height) = video_stream.start(&app)?;

    // Store in state
    {
        let mut stream = state.video_stream.lock().await;
        *stream = Some(video_stream);
    }

    info!(
        "Video stream started via Tauri events ({}x{})",
        width, height
    );

    Ok(StreamInfo {
        width,
        height,
        running: true,
        event_name: video_stream::VIDEO_FRAME_EVENT.to_string(),
    })
}

#[tauri::command]
async fn stop_video_stream(state: tauri::State<'_, AppState>) -> Result<(), String> {
    info!("Stopping video stream");

    let mut stream = state.video_stream.lock().await;
    if let Some(ref mut s) = *stream {
        s.stop();
    }
    *stream = None;

    info!("Video stream stopped");
    Ok(())
}

#[tauri::command]
async fn get_stream_info(state: tauri::State<'_, AppState>) -> Result<Option<StreamInfo>, String> {
    let stream = state.video_stream.lock().await;

    match &*stream {
        Some(s) => {
            let (width, height) = s.get_dimensions();
            Ok(Some(StreamInfo {
                width,
                height,
                running: s.is_running(),
                event_name: video_stream::VIDEO_FRAME_EVENT.to_string(),
            }))
        }
        None => Ok(None),
    }
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
        scrcpy_server: create_scrcpy_server(String::new()),
        video_stream: Arc::new(Mutex::new(None)),
        ws_scrcpy_server: create_ws_scrcpy_server(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_status,
            get_agent_logs,
            start_agent,
            stop_agent,
            save_config,
            open_web_ui,
            open_url,
            install_device_agent,
            install_adb,
            get_device_agent_status,
            sign_in,
            sign_in_oauth,
            sign_in_manual,
            sign_out,
            get_user_info,
            get_user_info_full,
            get_devices,
            get_registered_devices,
            update_device_name,
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
            // scrcpy-server Tauri events mode (for #mirror-screen div)
            start_scrcpy_mirror,
            stop_scrcpy_mirror,
            // Legacy scrcpy-server commands
            start_mirror,
            stop_mirror,
            mirror_control,
            mirror_tap,
            mirror_swipe,
            // Video stream commands
            start_video_stream,
            stop_video_stream,
            get_stream_info,
            // Mirror dependency management
            install_mirror_deps,
            get_mirror_deps_status,
            start_ws_scrcpy_server,
            stop_ws_scrcpy_server,
            get_ws_scrcpy_status,
            get_install_mode,
        ])
        .setup(|app| {
            info!("Setting up system tray...");

            // Create tray menu
            let show_item = MenuItem::new(app, "Show AMOS Companion", true, None::<&str>)?;
            let open_web_item = MenuItem::new(app, "Open AMOS Web UI", true, None::<&str>)?;
            let quit_item = MenuItem::new(app, "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_item, &open_web_item, &quit_item])?;

            // Build tray icon
            let _tray = TrayIconBuilder::new()
                .tooltip("AMOS Companion")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "Show AMOS Companion" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "Open AMOS Web UI" => {
                            let _ = tauri_plugin_shell::ShellExt::shell(app)
                                .open("https://app.amos.moo-vpn.online", None);
                        }
                        "Quit" => {
                            info!("Quit requested from tray menu");
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            info!("System tray configured successfully");
            info!("Tauri app setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
