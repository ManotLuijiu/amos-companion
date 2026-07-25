use tracing::{error, info, warn};

/// Get or create the default workspace for a user
/// This is called during the auto-setup flow
pub async fn ensure_workspace_exists(
    api_url: &str,
) -> Result<String, String> {
    info!("Checking/creating default workspace at {}", api_url);
    
    // Call GET /auth/workspace/default - this creates if not exists
    let url = format!("{}/auth/workspace/default", api_url.trim_end_matches('/'));
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to AMOS API: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!("Failed to get/create workspace: {} - {}", status, body);
        return Err(format!("Failed to get/create workspace: {} - {}", status, body));
    }
    
    #[derive(serde::Deserialize)]
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
    label: &str,
) -> Result<(String, String, String), String> {
    info!("Registering device-agent: {} for workspace {}", label, workspace_id);
    
    let url = format!("{}/auth/device-agent/register", api_url.trim_end_matches('/'));
    
    #[derive(serde::Serialize)]
    struct RegisterRequest<'a> {
        label: &'a str,
        workspace_id: &'a str,
    }
    
    #[derive(serde::Deserialize)]
    struct RegisterResponse {
        api_key: String,
        api_secret: String,
        agent_id: String,
    }
    
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
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
        return Err(format!("Failed to register device-agent: {} - {}", status, body));
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
