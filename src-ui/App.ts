import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { TrayIcon } from "@tauri-apps/api/tray";
import { Menu, MenuItem } from "@tauri-apps/api/menu";
import { getCurrentWindow } from "@tauri-apps/api/window";

// ─── Types ────────────────────────────────────────────────────────────────────

interface AgentStatus {
	agent_online: boolean;
	agent_running: boolean;
	connected_devices: string[];
	platform: string;
	companion_version: string;
	adb_version: string;
	api_url: string;
	agent_pid: number | null;
	error_message: string | null;
}

interface DeviceInfo {
	serial: string;
	model: string;
	status: string;
	resolution: string | null;
	battery: number | null;
}

interface DeviceList {
	devices: DeviceInfo[];
}

type StatusDisplay = "loading" | "setup" | "stopped" | "running" | "error";

interface LogEntry {
	timestamp: Date;
	level: "info" | "warn" | "error" | "debug";
	message: string;
}

// ─── State ───────────────────────────────────────────────────────────────────

let state: AgentStatus = {
	agent_online: false,
	agent_running: false,
	connected_devices: [],
	platform: "",
	companion_version: "0.1.0",
	adb_version: "",
	api_url: "",
	agent_pid: null,
	error_message: null,
};

let display: StatusDisplay = "loading";
let selectedDevice: DeviceInfo | null = null;
let screenshotRefreshInterval: ReturnType<typeof setInterval> | null = null;
let scrcpyEnabled = false;
let scrcpyAvailable = false;
let deviceAgentInstalled = false;
let userInfo: { id: string; email: string } | null = null;
let currentMirroringDevice: string | null = null;
let logEntries: LogEntry[] = [];
const maxLogs = 500;

// ─── Logging ─────────────────────────────────────────────────────────────────

function addLog(level: LogEntry["level"], message: string): void {
	const entry: LogEntry = {
		timestamp: new Date(),
		level,
		message,
	};
	logEntries.push(entry);
	if (logEntries.length > maxLogs) {
		logEntries = logEntries.slice(-maxLogs);
	}
	appendLogToUI(entry);
}

function appendLogToUI(entry: LogEntry): void {
	const logContainer = document.getElementById("log-content");
	if (!logContainer) return;

	const time = entry.timestamp.toLocaleTimeString("en-US", {
		hour12: false,
		hour: "2-digit",
		minute: "2-digit",
		second: "2-digit",
	});

	const line = document.createElement("div");
	line.className = `log-line log-${entry.level}`;

	const timeSpan = document.createElement("span");
	timeSpan.className = "log-time";
	timeSpan.textContent = time;

	const levelSpan = document.createElement("span");
	levelSpan.className = `log-level log-level-${entry.level}`;
	levelSpan.textContent = entry.level.toUpperCase();

	const msgSpan = document.createElement("span");
	msgSpan.className = "log-message";
	msgSpan.textContent = entry.message;

	line.appendChild(timeSpan);
	line.appendChild(levelSpan);
	line.appendChild(msgSpan);
	logContainer.appendChild(line);

	// Auto-scroll to bottom
	logContainer.scrollTop = logContainer.scrollHeight;
}

function clearLogs(): void {
	logEntries = [];
	const logContainer = document.getElementById("log-content");
	if (logContainer) {
		logContainer.innerHTML = "";
	}
}

// ─── Tray Setup ───────────────────────────────────────────────────────────────

async function setupTray(): Promise<void> {
	try {
		const showItem = await MenuItem.new({
			id: "show",
			text: "Show AMOS Companion",
			action: async () => {
				const win = getCurrentWindow();
				await win.show();
				await win.setFocus();
			},
		});
		const openWebItem = await MenuItem.new({
			id: "open_web",
			text: "Open AMOS Web UI",
			action: async () => {
				await invoke("open_web_ui");
			},
		});
		const quitItem = await MenuItem.new({
			id: "quit",
			text: "Quit",
			action: async () => {
				const win = getCurrentWindow();
				await win.close();
			},
		});

		const menu = await Menu.new({
			items: [showItem, openWebItem, quitItem],
		});

		await TrayIcon.new({
			id: "main-tray",
			tooltip: "AMOS Companion",
			menu,
			showMenuOnLeftClick: false,
		});
	} catch (err) {
		addLog("error", `Failed to setup tray: ${err}`);
	}
}

// ─── Build UI ─────────────────────────────────────────────────────────────────

function build(): HTMLElement {
	const root = document.createElement("div");
	root.className = "app-container";

	root.append(buildHeader());
	root.append(buildLoginSection());
	root.append(buildMainContent());
	root.append(buildFooter());

	return root;
}

function buildLoginSection(): HTMLElement {
	const section = document.createElement("section");
	section.id = "login-section";
	section.className = "login-section";
	section.style.display = "none";

	const container = document.createElement("div");
	container.className = "login-container";

	const title = document.createElement("h2");
	title.textContent = "Sign in to AMOS";

	// API URL input
	const apiUrlLabel = document.createElement("label");
	apiUrlLabel.htmlFor = "login-api-url";
	apiUrlLabel.textContent = "API URL";

	const apiUrlInput = document.createElement("input");
	apiUrlInput.type = "url";
	apiUrlInput.id = "login-api-url";
	apiUrlInput.placeholder = "https://amos-api.moo-vpn.online";
	apiUrlInput.required = true;

	// Divider
	const divider = document.createElement("div");
	divider.className = "login-divider";
	const dividerText = document.createElement("span");
	dividerText.textContent = "or";
	divider.appendChild(dividerText);

	// Google OAuth button
	const googleBtn = document.createElement("button");
	googleBtn.type = "button";
	googleBtn.className = "btn btn-google";
	googleBtn.innerHTML =
		'<svg class="google-icon" viewBox="0 0 24 24"><path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z" fill="#4285F4"/><path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/><path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/><path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/></svg> Sign in with Google';
	googleBtn.addEventListener("click", handleGoogleLogin);

	// Email/Password form
	const divider2 = document.createElement("div");
	divider2.className = "login-divider";
	const dividerText2 = document.createElement("span");
	dividerText2.textContent = "or continue with email";
	divider2.appendChild(dividerText2);

	const form = document.createElement("form");
	form.id = "login-form";

	const emailLabel = document.createElement("label");
	emailLabel.htmlFor = "login-email";
	emailLabel.textContent = "Email";

	const emailInput = document.createElement("input");
	emailInput.type = "email";
	emailInput.id = "login-email";
	emailInput.placeholder = "your@email.com";
	emailInput.required = true;

	const passwordLabel = document.createElement("label");
	passwordLabel.htmlFor = "login-password";
	passwordLabel.textContent = "Password";

	const passwordInput = document.createElement("input");
	passwordInput.type = "password";
	passwordInput.id = "login-password";
	passwordInput.placeholder = "Password";
	passwordInput.required = true;

	const errorDiv = document.createElement("div");
	errorDiv.id = "login-error";
	errorDiv.className = "login-error";
	errorDiv.style.display = "none";

	const submitBtn = document.createElement("button");
	submitBtn.type = "submit";
	submitBtn.className = "btn btn-primary";
	submitBtn.textContent = "Sign In";

	form.append(
		emailLabel,
		emailInput,
		passwordLabel,
		passwordInput,
		errorDiv,
		submitBtn,
	);
	form.addEventListener("submit", handleLogin);

	container.append(title, googleBtn, divider, form);

	// Store API URL input for later use
	apiUrlInput.addEventListener("blur", () => {
		localStorage.setItem("login-api-url", apiUrlInput.value);
	});
	// Load saved API URL
	const savedApiUrl = localStorage.getItem("login-api-url");
	if (savedApiUrl) {
		apiUrlInput.value = savedApiUrl;
	}

	section.append(container);

	return section;
}

async function handleGoogleLogin(): Promise<void> {
	const apiUrlInput = document.getElementById(
		"login-api-url",
	) as HTMLInputElement;
	const apiUrl = apiUrlInput?.value || "https://amos-api.moo-vpn.online";
	const errorDiv = document.getElementById("login-error") as HTMLDivElement;
	const googleBtn = document.querySelector(".btn-google") as HTMLButtonElement;

	try {
		if (googleBtn) {
			googleBtn.disabled = true;
			googleBtn.textContent = "Opening browser...";
		}

		addLog("info", "Starting Google OAuth login...");

		// Use the new OAuth flow with local callback server
		await invoke("sign_in_oauth", { apiUrl });

		userInfo = { id: "", email: "" };

		const loginSection = document.getElementById("login-section");
		const mainContent = document.getElementById("main-content");
		if (loginSection) loginSection.style.display = "none";

		addLog("info", `Signed in successfully!`);
		await refreshStatus();
	} catch (e) {
		const errorMsg = e instanceof Error ? e.message : String(e);
		addLog("error", `Google sign-in failed: ${errorMsg}`);
		if (errorDiv) {
			errorDiv.textContent = errorMsg;
			errorDiv.style.display = "block";
		}
		if (googleBtn) {
			googleBtn.disabled = false;
			googleBtn.textContent = "Sign in with Google";
		}
	}
}

async function handleLogin(event: Event): Promise<void> {
	event.preventDefault();

	const form = event.target as HTMLFormElement;
	const emailInput = form.querySelector("#login-email") as HTMLInputElement;
	const passwordInput = form.querySelector(
		"#login-password",
	) as HTMLInputElement;
	const errorDiv = form.querySelector("#login-error") as HTMLDivElement;
	const submitBtn = form.querySelector(
		"button[type=submit]",
	) as HTMLButtonElement;

	// Get API URL from the input outside the form
	const apiUrlInput = document.getElementById(
		"login-api-url",
	) as HTMLInputElement;
	const apiUrl = apiUrlInput?.value || "https://amos-api.moo-vpn.online";
	const email = emailInput.value;
	const password = passwordInput.value;

	submitBtn.disabled = true;
	submitBtn.textContent = "Signing in...";
	errorDiv.style.display = "none";

	try {
		await invoke("sign_in", { apiUrl, email, password });
		userInfo = { id: "", email };

		const loginSection = document.getElementById("login-section");
		const mainContent = document.getElementById("main-content");
		if (loginSection) loginSection.style.display = "none";

		addLog("info", `Signed in as ${email}`);

		// Refresh status now that we're logged in
		await refreshStatus();
	} catch (e) {
		const errorMsg = e instanceof Error ? e.message : String(e);
		addLog("error", `Login failed: ${errorMsg}`);
		errorDiv.textContent = errorMsg;
		errorDiv.style.display = "block";
	} finally {
		submitBtn.disabled = false;
		submitBtn.textContent = "Sign In";
	}
}

function buildHeader(): HTMLElement {
	const header = document.createElement("header");
	header.className = "app-header";

	// Logo & Title
	const brand = document.createElement("div");
	brand.className = "header-brand";

	const logo = document.createElement("img");
	logo.className = "header-logo";
	logo.src = "amos-logo.png";
	logo.alt = "AMOS Logo";

	const titleGroup = document.createElement("div");
	titleGroup.className = "header-title-group";

	const title = document.createElement("h1");
	title.className = "header-title";
	title.textContent = "AMOS Companion";

	const version = document.createElement("span");
	version.className = "header-version";
	version.textContent = `v${state.companion_version}`;

	titleGroup.appendChild(title);
	titleGroup.appendChild(version);
	brand.appendChild(logo);
	brand.appendChild(titleGroup);

	// Status Badge
	const statusBadge = document.createElement("div");
	statusBadge.className = "header-status";
	statusBadge.id = "status-badge";
	statusBadge.textContent = "Loading...";

	header.appendChild(brand);
	header.appendChild(statusBadge);

	return header;
}

function buildMainContent(): HTMLElement {
	const main = document.createElement("main");
	main.className = "app-main";
	main.id = "main-content";

	// ============================================
	// LEFT PANEL - Agent & Devices (340px)
	// ============================================
	const leftPanel = document.createElement("div");
	leftPanel.className = "panel panel-left";

	// Agent Status Card
	const agentCard = createAgentCard();
	leftPanel.appendChild(agentCard);

	// Device List Card with Search
	const deviceCard = createDeviceCard();
	leftPanel.appendChild(deviceCard);

	// Settings Card
	const settingsCard = createSettingsCard();
	leftPanel.appendChild(settingsCard);

	main.appendChild(leftPanel);

	// ============================================
	// MIDDLE PANEL - Mirror (340px)
	// ============================================
	const mirrorPanel = document.createElement("div");
	mirrorPanel.className = "panel panel-mirror";

	const mirrorCard = createMirrorCard();
	mirrorPanel.appendChild(mirrorCard);

	main.appendChild(mirrorPanel);

	// ============================================
	// RIGHT PANEL - Activity Logs (flex)
	// ============================================
	const rightPanel = document.createElement("div");
	rightPanel.className = "panel panel-right";

	const logCard = createLogCard();
	rightPanel.appendChild(logCard);

	main.appendChild(rightPanel);

	return main;
}

/**
 * Create Agent Status Card
 */
function createAgentCard(): HTMLElement {
	const card = document.createElement("div");
	card.className = "card";
	card.id = "agent-card";

	const header = document.createElement("div");
	header.className = "card-header";
	header.innerHTML = `
		<div class="card-title">
			<span class="card-icon">🎯</span>
			Agent Status
		</div>
	`;
	card.appendChild(header);

	const body = document.createElement("div");
	body.className = "card-body";
	body.innerHTML = `
		<div class="status-display">
			<div class="status-indicator-large" id="status-indicator-large"></div>
			<div class="status-info">
				<div class="status-label-large" id="status-label-large">Loading...</div>
				<div class="status-detail" id="status-detail-text">Connecting...</div>
			</div>
		</div>
		<div class="status-meta" id="status-meta">
			<div class="meta-item">
				<span class="meta-label">PID</span>
				<span class="meta-value" id="meta-pid">—</span>
			</div>
			<div class="meta-item">
				<span class="meta-label">Platform</span>
				<span class="meta-value" id="meta-platform">—</span>
			</div>
			<div class="meta-item">
				<span class="meta-label">Devices</span>
				<span class="meta-value" id="meta-devices">0</span>
			</div>
		</div>
	`;
	card.appendChild(body);

	const actions = document.createElement("div");
	actions.className = "card-actions";
	actions.innerHTML = `
		<button class="btn btn-primary btn-large" id="btn-start">
			<span class="btn-icon">▶</span>
			Start Agent
		</button>
		<button class="btn btn-danger btn-large" id="btn-stop" disabled>
			<span class="btn-icon">■</span>
			Stop Agent
		</button>
	`;
	card.appendChild(actions);

	return card;
}

/**
 * Create Device List Card with Search
 */
function createDeviceCard(): HTMLElement {
	const card = document.createElement("div");
	card.className = "card";

	const header = document.createElement("div");
	header.className = "card-header";
	header.innerHTML = `
		<div class="card-title">
			<span class="card-icon">📱</span>
			Devices
		</div>
		<span class="badge badge-info" id="device-count">0</span>
	`;
	card.appendChild(header);

	const body = document.createElement("div");
	body.className = "card-body";
	body.style.padding = "12px";
	body.innerHTML = `
		<div class="device-search-container" style="margin-bottom: 12px;">
			<input type="text" id="device-search" class="setting-input" 
				placeholder="🔍 Search devices..." style="width: 100%;" />
		</div>
		<div class="device-list-container" id="device-list-container">
			<ul class="device-list" id="device-list">
				<li class="device-empty" id="device-empty">
					<span class="empty-icon">📲</span>
					<span>No devices connected</span>
				</li>
			</ul>
		</div>
	`;
	card.appendChild(body);

	return card;
}

/**
 * Create Settings Card
 */
function createSettingsCard(): HTMLElement {
	const card = document.createElement("div");
	card.className = "card";

	const header = document.createElement("div");
	header.className = "card-header collapsible";
	header.innerHTML = `
		<div class="card-title">
			<span class="card-icon">⚙️</span>
			Settings
		</div>
		<span class="collapse-icon">▼</span>
	`;
	header.onclick = () => card.classList.toggle("collapsed");
	card.appendChild(header);

	const body = document.createElement("div");
	body.className = "card-body";
	body.innerHTML = `
		<div class="setting-item">
			<label class="setting-label">AMOS API URL</label>
			<input type="url" class="setting-input" id="api-url" placeholder="https://amos-api.moo-vpn.online" />
		</div>
		<div class="setting-item">
			<label class="setting-label">
				High Performance Mode
				<span class="setting-hint" id="scrcpy-status"></span>
			</label>
			<div class="toggle-container">
				<input type="checkbox" id="toggle-scrcpy" class="toggle-input" />
				<label for="toggle-scrcpy" class="toggle-label"></label>
				<span class="toggle-text" id="toggle-scrcpy-text">Requires scrcpy</span>
			</div>
		</div>
		<button class="btn btn-secondary btn-full" id="btn-open-web">
			🌐 Open AMOS Web UI
		</button>
	`;
	card.appendChild(body);

	return card;
}

/**
 * Create Mirror Card (new dedicated panel)
 */
function createMirrorCard(): HTMLElement {
	const card = document.createElement("div");
	card.className = "mirror-card";
	card.id = "mirror-card";

	const header = document.createElement("div");
	header.className = "mirror-header";
	header.innerHTML = `
		<div class="mirror-title">
			<span>📺</span>
			<span class="device-name" id="mirror-device-name">Screen Mirror</span>
		</div>
		<button class="mirror-close" id="btn-close-mirror" title="Close mirror">✕</button>
	`;
	card.appendChild(header);

	const screenContainer = document.createElement("div");
	screenContainer.className = "mirror-screen-container";
	screenContainer.id = "mirror-screen-container";
	screenContainer.innerHTML = `
		<div class="mirror-screen-placeholder" id="mirror-placeholder">
			<span class="icon">📱</span>
			<span class="text">Select a device to start mirroring</span>
		</div>
		<div class="mirror-loading" id="mirror-loading" style="display: none;">
			<div class="spinner"></div>
			<span>Connecting to device...</span>
		</div>
		<img class="mirror-screen" id="mirror-screen" style="display: none;" alt="Device Screen" />
	`;
	card.appendChild(screenContainer);

	const controls = document.createElement("div");
	controls.className = "mirror-controls";
	controls.id = "mirror-controls";
	controls.style.display = "none";
	controls.innerHTML = `
		<button class="btn btn-secondary" id="btn-mirror-back" title="Back">⬅</button>
		<button class="btn btn-secondary" id="btn-mirror-home" title="Home">🏠</button>
		<button class="btn btn-secondary" id="btn-mirror-enter" title="Enter">↵</button>
		<button class="btn btn-secondary" id="btn-mirror-power" title="Power">⏻</button>
	`;
	card.appendChild(controls);

	const status = document.createElement("div");
	status.className = "mirror-status";
	status.id = "mirror-status";
	status.innerHTML = `
		<div class="status-item">
			<span>●</span>
			<span id="mirror-battery">—</span>
		</div>
		<div class="status-item">
			<span>📶</span>
			<span id="mirror-wifi">—</span>
		</div>
	`;
	card.appendChild(status);

	return card;
}

/**
 * Create Log Card
 */
function createLogCard(): HTMLElement {
	const card = document.createElement("div");
	card.className = "card card-logs";
	card.style.height = "100%";

	const header = document.createElement("div");
	header.className = "card-header";
	header.innerHTML = `
		<div class="card-title">
			<span class="card-icon">📋</span>
			Activity Logs
		</div>
		<div class="log-controls">
			<select id="log-filter" class="log-filter-select" style="
				background: var(--bg-tertiary);
				border: 1px solid var(--border-color);
				color: var(--text-primary);
				padding: 4px 8px;
				border-radius: 6px;
				font-size: 11px;
			">
				<option value="all">All</option>
				<option value="info">Info</option>
				<option value="warn">Warning</option>
				<option value="error">Error</option>
			</select>
			<button class="btn btn-small btn-ghost" id="btn-export-logs" title="Export logs">📤</button>
			<button class="btn btn-small btn-ghost" id="btn-clear-logs" title="Clear logs">🗑️</button>
		</div>
	`;
	card.appendChild(header);

	const body = document.createElement("div");
	body.className = "log-container";
	body.id = "log-container";
	body.style.flex = "1";
	body.innerHTML = `<div class="log-content" id="log-content"></div>`;
	card.appendChild(body);

	return card;
}

function buildFooter(): HTMLElement {
	const footer = document.createElement("footer");
	footer.className = "app-footer";
	footer.innerHTML = `
		<span>AMOS Device Management</span>
		<span class="footer-sep">•</span>
		<span>${new Date().getFullYear()}</span>
	`;
	return footer;
}

// ─── Event Handlers ────────────────────────────────────────────────────────────

async function handleStart(): Promise<void> {
	const btnStart = document.getElementById("btn-start") as HTMLButtonElement;

	if (btnStart) {
		btnStart.disabled = true;
		btnStart.textContent = "Starting...";
	}

	addLog("info", "Attempting to start agent...");

	try {
		await invoke("start_agent");
		addLog("info", "Agent start command sent successfully");

		// Wait for process to actually start
		await new Promise((resolve) => setTimeout(resolve, 1000));
		await refreshStatus();

		if (state.agent_running) {
			addLog("info", `Agent started with PID: ${state.agent_pid}`);
		}
	} catch (err) {
		addLog("error", `Failed to start agent: ${err}`);
		if (btnStart) {
			btnStart.disabled = false;
			btnStart.textContent = "Start Agent";
		}
	}
}

async function handleStop(): Promise<void> {
	const btnStop = document.getElementById("btn-stop") as HTMLButtonElement;

	if (btnStop) {
		btnStop.disabled = true;
		btnStop.textContent = "Stopping...";
	}

	addLog("info", "Attempting to stop agent...");

	try {
		await invoke("stop_agent");
		addLog("info", "Agent stop command sent successfully");

		await new Promise((resolve) => setTimeout(resolve, 500));
		await refreshStatus();

		if (!state.agent_running) {
			addLog("info", "Agent stopped successfully");
		}
	} catch (err) {
		addLog("error", `Failed to stop agent: ${err}`);
	}

	if (btnStop) {
		btnStop.disabled = true;
		btnStop.textContent = "Stop Agent";
	}
}

async function handleOpenWebUI(): Promise<void> {
	addLog("info", "Opening AMOS Web UI...");
	await invoke("open_web_ui");
}

async function handleDeviceClick(device: DeviceInfo): Promise<void> {
	addLog("info", `Opening control for: ${device.model} (${device.serial})`);
	selectedDevice = device;

	// Show device control panel
	const controlCard = document.getElementById("device-control-card");
	const deviceName = document.getElementById("control-device-name");
	if (controlCard) controlCard.style.display = "block";
	if (deviceName) deviceName.textContent = device.model;

	// Start screenshot refresh
	await refreshScreenshot();
	if (screenshotRefreshInterval) clearInterval(screenshotRefreshInterval);
	screenshotRefreshInterval = setInterval(refreshScreenshot, 1000);

	// Get device info
	try {
		const info = await invoke<DeviceInfo>("get_device_info", {
			serial: device.serial,
		});
		const infoBar = document.getElementById("device-info-bar");
		if (infoBar)
			infoBar.textContent = `${info.resolution || "Unknown"} | Battery: ${info.battery ?? "N/A"}%`;
	} catch (err) {
		addLog("warn", `Could not get device info: ${err}`);
	}
}

function closeDevicePanel(): void {
	selectedDevice = null;
	const controlCard = document.getElementById("device-control-card");
	if (controlCard) controlCard.style.display = "none";
	if (screenshotRefreshInterval) {
		clearInterval(screenshotRefreshInterval);
		screenshotRefreshInterval = null;
	}
}

let deviceErrorCount = 0;
let mirrorErrorCount = 0;
const MAX_SCREENSHOT_ERRORS = 3;

async function refreshScreenshot(): Promise<void> {
	if (!selectedDevice) return;
	const loading = document.getElementById("screen-loading");
	const screenImg = document.getElementById(
		"device-screen",
	) as HTMLImageElement;

	if (loading) loading.style.display = "flex";

	try {
		const base64 = await invoke<string>("capture_screenshot", {
			serial: selectedDevice.serial,
		});
		if (screenImg && base64) {
			screenImg.src = `data:image/png;base64,${base64}`;
		}
		deviceErrorCount = 0;
	} catch (err) {
		deviceErrorCount++;
		if (deviceErrorCount >= MAX_SCREENSHOT_ERRORS) {
			addLog(
				"error",
				"Device screenshot failed. Check USB debugging authorization.",
			);
			if (screenshotRefreshInterval) {
				clearInterval(screenshotRefreshInterval);
				screenshotRefreshInterval = null;
			}
		}
	}

	if (loading) loading.style.display = "none";
}

async function handleScreenTap(event: MouseEvent): Promise<void> {
	if (!currentMirroringDevice) return;

	const mirrorScreen = document.getElementById(
		"mirror-screen",
	) as HTMLImageElement;
	if (!mirrorScreen || !mirrorScreen.naturalWidth) return;

	const rect = mirrorScreen.getBoundingClientRect();
	const scaleX = mirrorScreen.naturalWidth / rect.width;
	const scaleY = mirrorScreen.naturalHeight / rect.height;
	const x = Math.round((event.clientX - rect.left) * scaleX);
	const y = Math.round((event.clientY - rect.top) * scaleY);

	try {
		await invoke("device_tap", { serial: currentMirroringDevice, x, y });
		addLog("debug", `Tap: ${x},${y}`);
	} catch (err) {
		addLog("error", `Tap failed: ${err}`);
	}
}

async function handleControlBack(): Promise<void> {
	if (!selectedDevice) return;
	try {
		await invoke("device_back", { serial: selectedDevice.serial });
		addLog("debug", "Back button sent");
	} catch (err) {
		addLog("error", `Back failed: ${err}`);
	}
}

async function handleControlHome(): Promise<void> {
	if (!selectedDevice) return;
	try {
		await invoke("device_home", { serial: selectedDevice.serial });
		addLog("debug", "Home button sent");
	} catch (err) {
		addLog("error", `Home failed: ${err}`);
	}
}

async function handleControlEnter(): Promise<void> {
	if (!selectedDevice) return;
	try {
		await invoke("device_enter", { serial: selectedDevice.serial });
		addLog("debug", "Enter button sent");
	} catch (err) {
		addLog("error", `Enter failed: ${err}`);
	}
}

async function handleScrcpyToggle(): Promise<void> {
	const toggle = document.getElementById("toggle-scrcpy") as HTMLInputElement;
	if (!toggle || !selectedDevice) return;

	if (toggle.checked) {
		if (!scrcpyAvailable) {
			addLog("warn", "scrcpy not found. Install with: brew install scrcpy");
			toggle.checked = false;
			return;
		}
		try {
			await invoke("start_scrcpy", { serial: selectedDevice.serial });
			scrcpyEnabled = true;
			addLog("info", `scrcpy started - check new window!`);
		} catch (err) {
			addLog("error", `Failed to start scrcpy: ${err}`);
			toggle.checked = false;
		}
	} else {
		try {
			await invoke("stop_scrcpy");
			scrcpyEnabled = false;
			addLog("info", "scrcpy stopped");
		} catch (err) {
			addLog("error", `Failed to stop scrcpy: ${err}`);
		}
	}
}

async function handleSaveConfig(): Promise<void> {
	const apiUrlInput = document.getElementById("api-url") as HTMLInputElement;
	if (apiUrlInput) {
		const apiUrl = apiUrlInput.value;
		try {
			await invoke("save_config", { apiUrl });
			addLog("info", `API URL saved: ${apiUrl}`);
		} catch (err) {
			addLog("error", `Failed to save config: ${err}`);
		}
	}
}

// ─── Status Refresh ──────────────────────────────────────────────────────────

async function refreshStatus(): Promise<void> {
	try {
		const status = await invoke<AgentStatus>("get_status");
		state = status;
		display = computeDisplay();
		refreshUI();

		// Refresh device list
		try {
			const deviceList = await invoke<DeviceList>("get_devices");
			refreshDeviceList(deviceList.devices);
		} catch (err) {
			addLog("debug", `Could not get device list: ${err}`);
		}
	} catch (err) {
		display = "setup";
		addLog("warn", "Could not connect to agent");
		refreshUI();
	}
}

function computeDisplay(): StatusDisplay {
	if (!state.platform) return "loading";
	if (state.error_message) return "error";
	if (state.agent_running) return "running";
	return "stopped";
}

function refreshUI(): void {
	// Status badge
	const statusBadge = document.getElementById("status-badge");
	if (statusBadge) {
		statusBadge.className = `header-status status-${display}`;
		statusBadge.textContent = getStatusLabel();
	}

	// Large status indicator
	const indicator = document.getElementById("status-indicator-large");
	if (indicator) {
		indicator.className = `status-indicator-large status-${display}`;
	}

	// Status label
	const label = document.getElementById("status-label-large");
	if (label) {
		label.textContent = getStatusLabel();
	}

	// Status detail
	const detail = document.getElementById("status-detail-text");
	if (detail) {
		detail.textContent = getStatusDetail();
	}

	// Meta info
	const metaPid = document.getElementById("meta-pid");
	const metaPlatform = document.getElementById("meta-platform");
	const metaDevices = document.getElementById("meta-devices");

	if (metaPid) metaPid.textContent = state.agent_pid?.toString() ?? "—";
	if (metaPlatform) metaPlatform.textContent = state.platform || "—";
	if (metaDevices)
		metaDevices.textContent = state.connected_devices.length.toString();

	// Buttons
	const btnStart = document.getElementById("btn-start") as HTMLButtonElement;
	const btnStop = document.getElementById("btn-stop") as HTMLButtonElement;

	if (btnStart && btnStop) {
		if (display === "running") {
			btnStart.disabled = true;
			btnStart.innerHTML = '<span class="btn-icon">✓</span> Running';
			btnStop.disabled = false;
		} else {
			btnStart.disabled = false;
			btnStart.innerHTML = '<span class="btn-icon">▶</span> Start Agent';
			btnStop.disabled = true;
		}
	}

	// API URL
	const apiUrlInput = document.getElementById("api-url") as HTMLInputElement;
	if (apiUrlInput && state.api_url) {
		apiUrlInput.value = state.api_url;
	}

	// Error message
	if (display === "error" && state.error_message) {
		addLog("error", state.error_message);
	}
}

function getStatusLabel(): string {
	switch (display) {
		case "loading":
			return "Connecting...";
		case "setup":
			return "Setup Required";
		case "stopped":
			return "Stopped";
		case "running":
			return "Running";
		case "error":
			return "Error";
		default:
			return "Unknown";
	}
}

function getStatusDetail(): string {
	switch (display) {
		case "loading":
			return "Initializing AMOS Companion...";
		case "setup":
			return "Configure API URL in settings";
		case "stopped":
			return "Click Start to begin";
		case "running":
			return state.agent_pid ? `PID: ${state.agent_pid}` : "Agent active";
		case "error":
			return state.error_message ?? "An error occurred";
		default:
			return "";
	}
}

function refreshDeviceList(devices: DeviceInfo[]): void {
	const list = document.getElementById("device-list");
	const empty = document.getElementById("device-empty");
	const count = document.getElementById("device-count");
	const searchInput = document.getElementById(
		"device-search",
	) as HTMLInputElement;
	const searchTerm = searchInput?.value?.toLowerCase() ?? "";

	// Filter devices by search term
	const filteredDevices = searchTerm
		? devices.filter(
				(d) =>
					d.model?.toLowerCase().includes(searchTerm) ||
					d.serial?.toLowerCase().includes(searchTerm) ||
					d.status?.toLowerCase().includes(searchTerm),
			)
		: devices;

	if (count) count.textContent = `${filteredDevices.length}/${devices.length}`;

	if (!list) return;

	// Clear existing items except empty state
	const items = list.querySelectorAll(".device-item");
	items.forEach((item) => item.remove());

	if (filteredDevices.length === 0) {
		if (empty) {
			empty.style.display = "flex";
			const emptySpan = empty.querySelector("span:last-child");
			if (emptySpan) {
				emptySpan.textContent = searchTerm
					? "No devices match your search"
					: "No devices connected";
			}
		}
	} else {
		if (empty) empty.style.display = "none";

		filteredDevices.forEach((device) => {
			const li = document.createElement("li");
			li.className = `device-item ${device.status === "connected" ? "device-online" : "device-offline"}`;
			const isMirroring = currentMirroringDevice === device.serial;
			li.innerHTML = `
				<span class="device-icon">${device.status === "connected" ? "●" : "○"}</span>
				<div class="device-info">
					<span class="device-name">${device.model || "Unknown Device"}</span>
					<span class="device-serial">${device.serial}</span>
				</div>
				<div class="device-actions">
					<button class="btn btn-small ${isMirroring ? "btn-primary" : "btn-secondary"} btn-mirror" 
						title="${isMirroring ? "Stop mirroring" : "Start mirroring"}" 
						data-device="${device.serial}">
						${isMirroring ? "■" : "👁️"}
					</button>
				</div>
			`;
			// Device click - select device
			li.onclick = (e) => {
				const target = e.target as HTMLElement;
				if (target.classList.contains("btn-mirror")) {
					e.stopPropagation();
					toggleDeviceMirror(device);
				} else {
					handleDeviceClick(device);
				}
			};
			list.appendChild(li);
		});
	}
}

// ─── Mirror Functions ──────────────────────────────────────────────────────────

async function toggleDeviceMirror(device: DeviceInfo): Promise<void> {
	if (currentMirroringDevice === device.serial) {
		// Stop mirroring
		await stopMirror();
	} else {
		// Start mirroring
		await startMirror(device);
	}
}

// Video streaming state
let videoStream: VideoStream | null = null;
let screenshotPollingInterval: ReturnType<typeof setInterval> | null = null;
const MIRROR_REFRESH_MS = 200; // 5 FPS fallback

// WebCodecs-based video player
class VideoStream {
	private ws: WebSocket | null = null;
	private decoder: globalThis.VideoDecoder | null = null;
	private canvas: HTMLCanvasElement | null = null;
	private ctx: CanvasRenderingContext2D | null = null;
	private running = false;
	private width = 1080;
	private height = 1920;

	constructor(private serial: string) {}

	async start(): Promise<void> {
		// Get stream info from Rust backend
		try {
			const info = await invoke<{
				ws_url: string;
				port: number;
				running: boolean;
			}>("start_video_stream", { serial: this.serial });

			if (!info.running) {
				throw new Error("Stream failed to start");
			}

			this.startWebSocket(info.ws_url);
		} catch (error) {
			addLog("error", `Failed to start video stream: ${error}`);
			throw error;
		}
	}

	private startWebSocket(wsUrl: string): void {
		this.running = true;

		// Create canvas for rendering
		this.canvas = document.createElement("canvas");
		this.canvas.width = this.width;
		this.canvas.height = this.height;
		this.ctx = this.canvas.getContext("2d");

		// Connect WebSocket
		this.ws = new WebSocket(wsUrl);

		this.ws.binaryType = "arraybuffer";

		this.ws.onopen = () => {
			addLog("info", "Video stream connected");
		};

		this.ws.onmessage = (event) => {
			if (event.data instanceof ArrayBuffer) {
				this.handleFrame(event.data);
			}
		};

		this.ws.onerror = (error) => {
			addLog("error", `WebSocket error: ${error}`);
		};

		this.ws.onclose = () => {
			if (this.running) {
				addLog("warn", "Video stream disconnected, retrying...");
				setTimeout(() => {
					if (this.running) {
						this.startWebSocket(wsUrl);
					}
				}, 1000);
			}
		};
	}

	private handleFrame(data: ArrayBuffer): void {
		// Try WebCodecs first (Chrome 94+)
		if (typeof VideoDecoder !== "undefined") {
			this.decodeWithWebCodecs(data);
		} else {
			// Fallback: try to display as PNG if possible
			this.displayAsImage(data);
		}
	}

	private async decodeWithWebCodecs(data: ArrayBuffer): Promise<void> {
		if (!this.decoder) {
			this.initDecoder();
		}

		if (!this.decoder || !this.canvas || !this.ctx) return;

		try {
			const chunk = new EncodedVideoChunk({
				type: "key", // screenrecord sends all keyframes
				timestamp: Date.now() * 1000,
				data,
			});

			this.decoder.decode(chunk);
		} catch (error) {
			// If decode fails, try as image
			this.displayAsImage(data);
		}
	}

	private initDecoder(): void {
		if (typeof VideoDecoder === "undefined") return;

		this.decoder = new VideoDecoder({
			output: (frame) => {
				if (this.canvas && this.ctx) {
					// Scale and draw frame to canvas
					this.ctx.drawImage(
						frame,
						0,
						0,
						this.canvas.width,
						this.canvas.height,
					);
					frame.close();

					// Update image element
					this.updateDisplay();
				}
			},
			error: (error) => {
				addLog("error", `Decoder error: ${error}`);
			},
		});

		this.decoder.configure({
			codec: "avc1.64001f", // H.264 High Profile Level 3.1
			codedWidth: this.width,
			codedHeight: this.height,
		});
	}

	private displayAsImage(data: ArrayBuffer): void {
		// Try to create image from raw data
		// This is a fallback for when h264 decoding isn't working
		try {
			const blob = new Blob([data], { type: "image/png" });
			const url = URL.createObjectURL(blob);

			const img = new Image();
			img.onload = () => {
				if (this.canvas && this.ctx) {
					this.ctx.drawImage(img, 0, 0, this.canvas.width, this.canvas.height);
					this.updateDisplay();
				}
				URL.revokeObjectURL(url);
			};
			img.src = url;
		} catch {
			// Silently ignore
		}
	}

	private updateDisplay(): void {
		const mirrorScreen = document.getElementById(
			"mirror-screen",
		) as HTMLImageElement;
		if (mirrorScreen && this.canvas) {
			mirrorScreen.src = this.canvas.toDataURL("image/png");
		}
	}

	stop(): void {
		this.running = false;

		if (this.ws) {
			this.ws.close();
			this.ws = null;
		}

		if (this.decoder) {
			this.decoder.close();
			this.decoder = null;
		}

		// Stop Rust backend stream
		invoke("stop_video_stream").catch(() => {});
	}
}

async function startMirror(device: DeviceInfo): Promise<void> {
	const mirrorScreen = document.getElementById(
		"mirror-screen",
	) as HTMLImageElement;
	const mirrorPlaceholder = document.getElementById("mirror-placeholder");
	const mirrorLoading = document.getElementById("mirror-loading");
	const mirrorControls = document.getElementById("mirror-controls");
	const mirrorDeviceName = document.getElementById("mirror-device-name");
	const mirrorBattery = document.getElementById("mirror-battery");
	const mirrorWifi = document.getElementById("mirror-wifi");

	if (!mirrorScreen) return;

	addLog("info", `Starting mirror for ${device.model || device.serial}...`);

	// Show loading
	if (mirrorPlaceholder) mirrorPlaceholder.style.display = "none";
	if (mirrorLoading) mirrorLoading.style.display = "flex";
	if (mirrorScreen) mirrorScreen.style.display = "none";
	if (mirrorControls) mirrorControls.style.display = "flex";
	if (mirrorDeviceName)
		mirrorDeviceName.textContent = device.model || device.serial;

	currentMirroringDevice = device.serial;

	// Stop any existing stream
	stopMirror();

	// Try WebSocket video streaming
	try {
		videoStream = new VideoStream(device.serial);
		await videoStream.start();

		if (mirrorScreen) mirrorScreen.style.display = "block";
		if (mirrorLoading) mirrorLoading.style.display = "none";
		addLog("info", `Mirror started for ${device.serial}`);
	} catch (error) {
		addLog("warn", `Video stream failed, using screenshot fallback: ${error}`);

		// Fallback to screenshot polling
		await refreshMirrorScreen(device.serial);

		screenshotPollingInterval = setInterval(async () => {
			if (currentMirroringDevice) {
				await refreshMirrorScreen(currentMirroringDevice);
			}
		}, MIRROR_REFRESH_MS);

		if (mirrorScreen) mirrorScreen.style.display = "block";
		if (mirrorLoading) mirrorLoading.style.display = "none";
		addLog("info", `Mirror started (fallback) for ${device.serial}`);
	}

	// Update device info
	if (mirrorBattery)
		mirrorBattery.textContent = device.battery ? `${device.battery}%` : "—";
	if (mirrorWifi) mirrorWifi.textContent = "—";
}

async function refreshMirrorScreen(serial: string): Promise<void> {
	const mirrorScreen = document.getElementById(
		"mirror-screen",
	) as HTMLImageElement;
	if (!mirrorScreen || !currentMirroringDevice) return;

	try {
		const base64 = await invoke<string>("capture_screenshot", { serial });
		mirrorScreen.src = `data:image/png;base64,${base64}`;
		mirrorErrorCount = 0; // Reset error count on success
	} catch (error) {
		mirrorErrorCount++;
		if (mirrorErrorCount >= MAX_SCREENSHOT_ERRORS) {
			addLog(
				"error",
				"Device screenshot failed, stopping mirror. Check USB debugging authorization.",
			);
			stopMirror();
		}
	}
}

async function stopMirror(): Promise<void> {
	if (!currentMirroringDevice) return;

	const deviceSerial = currentMirroringDevice;
	addLog("info", `Stopping mirror for ${deviceSerial}...`);

	// Stop video stream
	if (videoStream) {
		videoStream.stop();
		videoStream = null;
	}

	// Stop polling
	if (screenshotPollingInterval) {
		clearInterval(screenshotPollingInterval);
		screenshotPollingInterval = null;
	}

	// Reset mirror UI
	const mirrorScreen = document.getElementById(
		"mirror-screen",
	) as HTMLImageElement;
	const mirrorPlaceholder = document.getElementById("mirror-placeholder");
	const mirrorLoading = document.getElementById("mirror-loading");
	const mirrorControls = document.getElementById("mirror-controls");

	if (mirrorScreen) {
		mirrorScreen.src = "";
		mirrorScreen.style.display = "none";
	}
	if (mirrorPlaceholder) mirrorPlaceholder.style.display = "flex";
	if (mirrorLoading) mirrorLoading.style.display = "none";
	if (mirrorControls) mirrorControls.style.display = "none";

	currentMirroringDevice = null;

	addLog("info", `Mirror stopped for ${deviceSerial}`);

	// Refresh device list to update button state
	refreshDeviceList([]);
}

async function handleMirrorControl(action: string): Promise<void> {
	if (!currentMirroringDevice) return;

	const serial = currentMirroringDevice;

	try {
		switch (action) {
			case "back":
				await invoke("device_back", { serial });
				break;
			case "home":
				await invoke("device_home", { serial });
				break;
			case "enter":
				await invoke("device_enter", { serial });
				break;
			default:
				addLog("warn", `Unknown mirror action: ${action}`);
				return;
		}
		addLog("info", `Mirror control: ${action}`);
	} catch (error) {
		addLog("error", `Mirror control failed: ${error}`);
	}
}

// ─── Setup Event Listeners ────────────────────────────────────────────────────

function setupEventListeners(): void {
	// Start button
	const btnStart = document.getElementById("btn-start");
	btnStart?.addEventListener("click", handleStart);

	// Stop button
	const btnStop = document.getElementById("btn-stop");
	btnStop?.addEventListener("click", handleStop);

	// Open Web UI button
	const btnOpenWeb = document.getElementById("btn-open-web");
	btnOpenWeb?.addEventListener("click", handleOpenWebUI);

	// API URL input - save on blur
	const apiUrlInput = document.getElementById("api-url") as HTMLInputElement;
	apiUrlInput?.addEventListener("blur", handleSaveConfig);
	apiUrlInput?.addEventListener("keydown", (e) => {
		if (e.key === "Enter") {
			(apiUrlInput as HTMLInputElement).blur();
		}
	});

	// Clear logs button
	const btnClearLogs = document.getElementById("btn-clear-logs");
	btnClearLogs?.addEventListener("click", () => {
		clearLogs();
		addLog("info", "Logs cleared");
	});

	// Device control buttons
	const btnCloseControl = document.getElementById("btn-close-control");
	btnCloseControl?.addEventListener("click", closeDevicePanel);

	const btnTap = document.getElementById("btn-tap");
	btnTap?.addEventListener("click", handleControlBack);

	const btnHome = document.getElementById("btn-home");
	btnHome?.addEventListener("click", handleControlHome);

	const btnEnter = document.getElementById("btn-enter");
	btnEnter?.addEventListener("click", handleControlEnter);

	// Screen tap handler
	const screenImg = document.getElementById("device-screen");
	screenImg?.addEventListener("click", (e: Event) =>
		handleScreenTap(e as MouseEvent),
	);

	// scrcpy toggle
	const toggleScrcpy = document.getElementById(
		"toggle-scrcpy",
	) as HTMLInputElement;
	toggleScrcpy?.addEventListener("change", handleScrcpyToggle);

	// Device search
	const deviceSearch = document.getElementById(
		"device-search",
	) as HTMLInputElement;
	deviceSearch?.addEventListener("input", () => {
		refreshDeviceList([]); // Will re-filter with current search term
	});

	// Mirror controls
	const btnCloseMirror = document.getElementById("btn-close-mirror");
	btnCloseMirror?.addEventListener("click", () => {
		stopMirror();
	});

	const btnMirrorBack = document.getElementById("btn-mirror-back");
	btnMirrorBack?.addEventListener("click", () => {
		handleMirrorControl("back");
	});

	const btnMirrorHome = document.getElementById("btn-mirror-home");
	btnMirrorHome?.addEventListener("click", () => {
		handleMirrorControl("home");
	});

	const btnMirrorEnter = document.getElementById("btn-mirror-enter");
	btnMirrorEnter?.addEventListener("click", () => {
		handleMirrorControl("enter");
	});

	const btnMirrorPower = document.getElementById("btn-mirror-power");
	btnMirrorPower?.addEventListener("click", () => {
		handleMirrorControl("power");
	});
}

// ─── Entry ───────────────────────────────────────────────────────────────────

export async function init(): Promise<void> {
	addLog("info", "AMOS Companion initializing...");

	// Set up system tray
	await setupTray();
	addLog("info", "System tray configured");

	// Build and mount UI
	const app = document.getElementById("app")!;
	app.appendChild(build());
	setupEventListeners();

	// Check if user is logged in
	try {
		const user = await invoke<[string, string] | null>("get_user_info");
		if (user) {
			userInfo = { id: user[0], email: user[1] };
			addLog("info", `Logged in as ${userInfo.email}`);
			const loginSection = document.getElementById("login-section");
			const mainContent = document.getElementById("main-content");
			if (loginSection) loginSection.style.display = "none";
		} else {
			addLog("info", "Please sign in to continue");
			const loginSection = document.getElementById("login-section");
			const mainContent = document.getElementById("main-content");
			if (loginSection) loginSection.style.display = "flex";
			if (mainContent) mainContent.style.display = "none";
		}
	} catch (e) {
		addLog("warn", `Could not check login status: ${e}`);
	}

	// Listen for status updates from Rust backend
	await listen<AgentStatus>("status-update", (event) => {
		state = event.payload;
		addLog("debug", `Status update received: running=${state.agent_running}`);
		refreshUI();
	});

	// Initial status fetch
	addLog("info", "Fetching initial status...");
	await refreshStatus();
	addLog("info", "Status refresh complete");

	// Check if scrcpy is available
	try {
		scrcpyAvailable = await invoke<boolean>("is_scrcpy_available_cmd");
		const statusEl = document.getElementById("scrcpy-status");
		if (statusEl) {
			statusEl.textContent = scrcpyAvailable
				? "✓ Available"
				: "✗ Not installed";
			statusEl.style.color = scrcpyAvailable
				? "var(--accent-green)"
				: "var(--accent-red)";
		}
		addLog("info", `scrcpy ${scrcpyAvailable ? "available" : "not found"}`);
	} catch {
		addLog("warn", "Could not check scrcpy availability");
	}

	// Check device agent installation status
	try {
		const agentStatus = await invoke<{
			installed: boolean;
			path: string;
			os: string;
		}>("get_device_agent_status");
		deviceAgentInstalled = agentStatus.installed;
		addLog(
			"info",
			`Device agent ${agentStatus.installed ? "installed" : "not found"} (${agentStatus.os})`,
		);
	} catch {
		addLog("warn", "Could not check device agent status");
	}

	// Auto-refresh status every 5 seconds
	setInterval(() => refreshStatus(), 5000);
}
