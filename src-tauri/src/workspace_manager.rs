use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

/// Sign in with email and password using better-auth API
/// Returns (user_id, email) on success
pub async fn sign_in(
    api_url: &str,
    email: &str,
    password: &str,
) -> Result<(String, String), String> {
    info!("Signing in as {}", email);

    let url = format!(
        "{}/api/auth/sign-in/email-password",
        api_url.trim_end_matches('/')
    );

    #[derive(Serialize)]
    struct SignInRequest<'a> {
        email: &'a str,
        password: &'a str,
    }

    #[derive(Deserialize)]
    struct SessionResponse {
        user: UserResponse,
    }

    #[derive(Deserialize)]
    struct UserResponse {
        id: String,
        email: String,
        email_verified: bool,
    }

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&SignInRequest { email, password })
        .send()
        .await
        .map_err(|e| format!("Failed to connect to AMOS API: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!("Sign in failed: {} - {}", status, body);
        return Err(format!("Sign in failed: {} - {}", status, body));
    }

    let session: SessionResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse sign-in response: {}", e))?;

    info!(
        "Sign in successful: {} ({})",
        session.user.email, session.user.id
    );
    Ok((session.user.id, session.user.email))
}

/// Get or create the default workspace for a user
/// This is called during the auto-setup flow
pub async fn ensure_workspace_exists(api_url: &str, user_id: &str) -> Result<String, String> {
    info!(
        "Checking/creating default workspace at {} for user {}",
        api_url, user_id
    );

    // Call GET /auth/workspace/default - this creates if not exists
    let url = format!("{}/auth/workspace/default", api_url.trim_end_matches('/'));

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("X-User-ID", user_id)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to AMOS API: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!("Failed to get/create workspace: {} - {}", status, body);
        return Err(format!(
            "Failed to get/create workspace: {} - {}",
            status, body
        ));
    }

    #[derive(Deserialize)]
    struct WorkspaceResponse {
        id: String,
        name: String,
    }

    let workspace: WorkspaceResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse workspace response: {}", e))?;

    info!("Workspace ready: {} ({})", workspace.name, workspace.id);
    Ok(workspace.id)
}

/// Register this device-agent with AMOS API
/// Returns (api_key, api_secret, agent_id)
pub async fn register_device_agent(
    api_url: &str,
    workspace_id: &str,
    user_id: &str,
    label: &str,
) -> Result<(String, String, String), String> {
    info!(
        "Registering device-agent: {} for workspace {} (user: {})",
        label, workspace_id, user_id
    );

    let url = format!(
        "{}/auth/device-agent/register",
        api_url.trim_end_matches('/')
    );

    #[derive(Serialize)]
    struct RegisterRequest<'a> {
        label: &'a str,
        workspace_id: &'a str,
    }

    #[derive(Deserialize)]
    struct RegisterResponse {
        api_key: String,
        api_secret: String,
        agent_id: String,
    }

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("X-User-ID", user_id)
        .json(&RegisterRequest {
            label,
            workspace_id,
        })
        .send()
        .await
        .map_err(|e| format!("Failed to register device-agent: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!("Failed to register device-agent: {} - {}", status, body);
        return Err(format!(
            "Failed to register device-agent: {} - {}",
            status, body
        ));
    }

    let result: RegisterResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse register response: {}", e))?;

    info!("Device-agent registered: agent_id={}", result.agent_id);
    Ok((result.api_key, result.api_secret, result.agent_id))
}

/// Get bearer token for device-agent
pub async fn get_device_token(
    api_url: &str,
    api_key: &str,
    api_secret: &str,
) -> Result<(String, i64), String> {
    info!("Getting device token from {}", api_url);

    let url = format!("{}/auth/device-token", api_url.trim_end_matches('/'));

    #[derive(Serialize)]
    struct TokenRequest<'a> {
        api_key: &'a str,
        api_secret: &'a str,
    }

    #[derive(Deserialize)]
    struct TokenResponse {
        token: String,
        expires_in: i64,
    }

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&TokenRequest {
            api_key,
            api_secret,
        })
        .send()
        .await
        .map_err(|e| format!("Failed to get device token: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        warn!("Failed to get device token: {} - {}", status, body);
        return Err(format!("Failed to get device token: {} - {}", status, body));
    }

    let result: TokenResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    info!("Token received, expires in {} seconds", result.expires_in);
    Ok((result.token, result.expires_in))
}

/// Get OS hostname for labeling
pub fn get_hostname() -> String {
    hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string())
}

/// Registered device info from the backend API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredDevice {
    pub id: String,
    pub name: String,
    pub adb_serial: String,
}

/// Get registered devices for the workspace (with device_id, name, serial)
pub async fn get_registered_devices(
    api_url: &str,
    user_id: &str,
    workspace_id: &str,
) -> Result<Vec<RegisteredDevice>, String> {
    info!("Getting registered devices from {} for user {} in workspace {}", api_url, user_id, workspace_id);

    let url = format!("{}/devices", api_url.trim_end_matches('/'));

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    info!("[API] GET {} with headers: X-User-ID={}, X-Workspace-ID={}", url, user_id, workspace_id);
    let response = client
        .get(&url)
        .header("X-User-ID", user_id)
        .header("X-Workspace-ID", workspace_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get devices: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        warn!("Failed to get devices: {} - {}", status, body);
        return Err(format!("Failed to get devices: {}", status));
    }

    let devices: Vec<RegisteredDevice> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse devices response: {}", e))?;

    info!("Got {} registered devices", devices.len());
    Ok(devices)
}

/// Update device name in the backend
pub async fn update_device_name(
    api_url: &str,
    user_id: &str,
    workspace_id: &str,
    device_id: &str,
    name: &str,
) -> Result<(), String> {
    info!("Updating device {} name to '{}' for user {} in workspace {}", device_id, name, user_id, workspace_id);

    let url = format!("{}/devices/{}", api_url.trim_end_matches('/'), device_id);

    #[derive(Serialize)]
    struct UpdateRequest<'a> {
        name: &'a str,
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    info!("[API] PATCH {} with headers: X-User-ID={}, X-Workspace-ID={}, body={{name: '{}'}}", url, user_id, workspace_id, name);
    let response = client
        .patch(&url)
        .header("X-User-ID", user_id)
        .header("X-Workspace-ID", workspace_id)
        .json(&UpdateRequest { name })
        .send()
        .await
        .map_err(|e| format!("Failed to update device: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!("Failed to update device: {} - {}", status, body);
        return Err(format!("Failed to update device: {}", status));
    }

    info!("Device {} name updated to '{}'", device_id, name);
    Ok(())
}
