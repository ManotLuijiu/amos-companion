//! AMOS Companion - Frontend Application
//!
//! ✅ WORKING FEATURES:
//! - Agent status display and control (start/stop)
//! - Device list with search and friendly names
//! - Built-in mirror panel (#mirror-screen div)
//!   - ADB video stream via screenrecord + WebCodecs
//!   - Automatic fallback to screenshot polling
//!   - 5-second timeout prevents false success state
//! - Tap/swipe gestures via pointer events
//! - Device control buttons (back, home, enter, power)
//! - OAuth and email/password login
//! - System tray integration
//! - Settings panel with API URL and scrcpy toggle
//!
//! ❌ NEEDS FIXING:
//! - scrcpy native binary mode: displays in separate macOS window
//! - scrcpy-server WebSocket mode: NOT wired up to UI yet
//!
//! For scrcpy integration into #mirror-screen, see scrcpy_server.rs

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

declare const __APP_VERSION__: string;

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

/**
 * Result returned from backend `start_scrcpy` Tauri command.
 * Mirrors the Rust struct `ScrcpyLaunchResult`.
 */
interface ScrcpyLaunchResult {
	success: boolean;
	pid: number | null;
	window_title: string;
	message: string;
	focus_attempted: boolean;
	focus_succeeded: boolean | null;
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
	companion_version: __APP_VERSION__,
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
let scrcpyServerStream: ScrcpyVideoStream | null = null;
let deviceAgentInstalled = false;
const relayEnabled = false;
const relayStatus: "disconnected" | "connecting" | "connected" | "error" =
	"disconnected";
let userInfo: { id: string; email: string } | null = null;
let currentMirroringDevice: string | null = null;
let logEntries: LogEntry[] = [];
const maxLogs = 500;

// ─── Device Friendly Names ───────────────────────────────────────────────────

const DEVICE_NAMES_KEY = "amos-device-names";

function getDeviceFriendlyName(serial: string): string | null {
	try {
		const names = JSON.parse(localStorage.getItem(DEVICE_NAMES_KEY) || "{}");
		return names[serial] || null;
	} catch {
		return null;
	}
}

function setDeviceFriendlyName(serial: string, name: string): void {
	try {
		const names = JSON.parse(localStorage.getItem(DEVICE_NAMES_KEY) || "{}");
		names[serial] = name.trim() || "";
		localStorage.setItem(DEVICE_NAMES_KEY, JSON.stringify(names));
	} catch {
		// Ignore storage errors
	}
}

function getDeviceDisplayName(device: DeviceInfo): string {
	return getDeviceFriendlyName(device.serial) || device.model || device.serial;
}

/**
 * Start inline editing of device friendly name
 */
function startDeviceNameEdit(): void {
	if (!currentMirroringDevice) return;

	const deviceNameSpan = document.getElementById("mirror-device-name");
	if (!deviceNameSpan || deviceNameSpan.querySelector("input")) return; // Already editing

	const currentName =
		getDeviceFriendlyName(currentMirroringDevice) || currentMirroringDevice;

	const input = document.createElement("input");
	input.type = "text";
	input.className = "device-name-input";
	input.value = currentName;
	input.maxLength = 30;

	// Replace span content with input
	deviceNameSpan.textContent = "";
	deviceNameSpan.appendChild(input);
	input.focus();
	input.select();

	const finishEdit = () => {
		const newName = input.value.trim();
		if (newName && newName !== currentName) {
			setDeviceFriendlyName(currentMirroringDevice!, newName);
			addLog("debug", `Device renamed to: ${newName}`);
		}
		// Restore display - use friendly name or serial
		deviceNameSpan.textContent =
			getDeviceFriendlyName(currentMirroringDevice!) || currentMirroringDevice!;
	};

	input.addEventListener("blur", finishEdit);
	input.addEventListener("keydown", (e) => {
		if (e.key === "Enter") {
			input.blur();
		} else if (e.key === "Escape") {
			input.value = currentName; // Cancel
			input.blur();
		}
	});
}

function getCompanionVersionLabel(): string {
	const version =
		state.companion_version && state.companion_version !== "0.0.0"
			? state.companion_version
			: __APP_VERSION__;
	return `v${version}`;
}

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
		logContainer.replaceChildren();
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
	// Uses the official Google wordmark downloaded from Google's brand
	// site (companion/src-ui/public/google-logo.png). Per Google's brand
	// guidelines we do not modify the asset. The wordmark is displayed in
	// its intended proportions and the clickable area triggers OAuth.
	const googleBtn = document.createElement("button");
	googleBtn.type = "button";
	googleBtn.className = "btn btn-google";
	const googleLogo = document.createElement("img");
	googleLogo.src = "google-logo.png";
	googleLogo.alt = "Google";
	googleLogo.className = "google-logo";
	googleLogo.setAttribute("loading", "lazy");
	googleBtn.append(googleLogo);
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
		updateUserBadgeFull();

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
	logo.src = "./amos-logo.png";
	logo.alt = "AMOS Logo";

	const titleGroup = document.createElement("div");
	titleGroup.className = "header-title-group";

	const title = document.createElement("h1");
	title.className = "header-title";
	title.textContent = "AMOS Companion";

	const version = document.createElement("span");
	version.className = "header-version";
	version.id = "header-version";
	version.textContent = getCompanionVersionLabel();

	titleGroup.appendChild(title);
	titleGroup.appendChild(version);
	brand.appendChild(logo);
	brand.appendChild(titleGroup);

	// Status Badge
	const statusBadge = document.createElement("div");
	statusBadge.className = "header-status";
	statusBadge.id = "status-badge";
	statusBadge.textContent = "Loading...";

	// User Badge
	const userBadge = document.createElement("div");
	userBadge.className = "header-user";
	userBadge.id = "header-user";
	userBadge.style.display = "flex";
	userBadge.style.flexDirection = "column";
	userBadge.style.alignItems = "flex-end";
	userBadge.style.lineHeight = "1.2";
	const userEmail = document.createElement("span");
	userEmail.id = "header-user-email";
	userEmail.textContent = "Not logged in";
	const userWorkspace = document.createElement("span");
	userWorkspace.id = "header-user-workspace";
	userWorkspace.style.fontSize = "10px";
	userWorkspace.style.opacity = "0.7";
	userWorkspace.textContent = "";
	userBadge.appendChild(userEmail);
	userBadge.appendChild(userWorkspace);

	header.appendChild(brand);
	header.appendChild(statusBadge);
	header.appendChild(userBadge);

	return header;
}

function updateUserBadge(): void {
	const userEmail = document.getElementById("header-user-email");
	const userWorkspace = document.getElementById("header-user-workspace");
	const userBadge = document.getElementById("header-user");
	if (userEmail && userWorkspace && userBadge) {
		if (userInfo) {
			userEmail.textContent = userInfo.email;
			userBadge.className = "header-user logged-in";
		} else {
			userEmail.textContent = "Not logged in";
			userWorkspace.textContent = "";
			userBadge.className = "header-user";
		}
	}
}

async function updateUserBadgeFull(): Promise<void> {
	try {
		const info = await invoke<[string, string, string] | null>("get_user_info_full");
		const userEmailEl = document.getElementById("header-user-email");
		const userWorkspaceEl = document.getElementById("header-user-workspace");
		const userBadgeEl = document.getElementById("header-user");

		if (info && userEmailEl && userWorkspaceEl && userBadgeEl) {
			const [userId, email, workspaceId] = info;
			const shortUserId = userId.substring(0, 8);
			const shortWsId = workspaceId.substring(0, 8);
			userEmailEl.textContent = `${email} (${shortUserId})`;
			userWorkspaceEl.textContent = `default workspace ${shortWsId}...`;
			userBadgeEl.className = "header-user logged-in";
		}
	} catch (err) {
		console.error("Failed to get user info:", err);
	}
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
	const title = document.createElement("div");
	title.className = "card-title";
	const icon = document.createElement("span");
	icon.className = "card-icon";
	icon.textContent = "🎯";
	title.append(icon, "Agent Status");
	header.appendChild(title);
	card.appendChild(header);

	const body = document.createElement("div");
	body.className = "card-body";
	const statusDisplay = document.createElement("div");
	statusDisplay.className = "status-display";
	const indicator = document.createElement("div");
	indicator.className = "status-indicator-large";
	indicator.id = "status-indicator-large";
	const statusInfo = document.createElement("div");
	statusInfo.className = "status-info";
	const statusLabel = document.createElement("div");
	statusLabel.className = "status-label-large";
	statusLabel.id = "status-label-large";
	statusLabel.textContent = "Loading...";
	const statusDetail = document.createElement("div");
	statusDetail.className = "status-detail";
	statusDetail.id = "status-detail-text";
	statusDetail.textContent = "Connecting...";
	statusInfo.append(statusLabel, statusDetail);
	statusDisplay.append(indicator, statusInfo);

	const statusMeta = document.createElement("div");
	statusMeta.className = "status-meta";
	statusMeta.id = "status-meta";
	[
		["PID", "meta-pid", "—"],
		["Platform", "meta-platform", "—"],
		["Devices", "meta-devices", "0"],
	].forEach(([label, id, value]) => {
		const item = document.createElement("div");
		item.className = "meta-item";
		const labelSpan = document.createElement("span");
		labelSpan.className = "meta-label";
		labelSpan.textContent = label;
		const valueSpan = document.createElement("span");
		valueSpan.className = "meta-value";
		valueSpan.id = id;
		valueSpan.textContent = value;
		item.append(labelSpan, valueSpan);
		statusMeta.appendChild(item);
	});
	body.append(statusDisplay, statusMeta);
	card.appendChild(body);

	const actions = document.createElement("div");
	actions.className = "card-actions";
	const btnStart = document.createElement("button");
	btnStart.className = "btn btn-primary btn-large";
	btnStart.id = "btn-start";
	const startIcon = document.createElement("span");
	startIcon.className = "btn-icon";
	startIcon.textContent = "▶";
	btnStart.append(startIcon, "Start Agent");
	const btnStop = document.createElement("button");
	btnStop.className = "btn btn-danger btn-large";
	btnStop.id = "btn-stop";
	btnStop.disabled = true;
	const stopIcon = document.createElement("span");
	stopIcon.className = "btn-icon";
	stopIcon.textContent = "■";
	btnStop.append(stopIcon, "Stop Agent");
	actions.append(btnStart, btnStop);
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
	const title = document.createElement("div");
	title.className = "card-title";
	const icon = document.createElement("span");
	icon.className = "card-icon";
	icon.textContent = "📱";
	title.append(icon, "Devices");
	const count = document.createElement("span");
	count.className = "badge badge-info";
	count.id = "device-count";
	count.textContent = "0";

	// Sync button with SVG icon
	const syncBtn = document.createElement("button");
	syncBtn.className = "btn-small btn-sync";
	syncBtn.id = "btn-sync-devices";
	syncBtn.title = "Sync Devices";
	syncBtn.textContent = "";
	const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
	svg.setAttribute("width", "14");
	svg.setAttribute("height", "14");
	svg.setAttribute("viewBox", "0 0 24 24");
	svg.setAttribute("fill", "none");
	svg.setAttribute("stroke", "currentColor");
	svg.setAttribute("stroke-width", "2");
	svg.setAttribute("stroke-linecap", "round");
	svg.setAttribute("stroke-linejoin", "round");
	const path1 = document.createElementNS("http://www.w3.org/2000/svg", "path");
	path1.setAttribute("d", "M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8");
	const path2 = document.createElementNS("http://www.w3.org/2000/svg", "path");
	path2.setAttribute("d", "M21 3v5h-5");
	svg.appendChild(path1);
	svg.appendChild(path2);
	syncBtn.appendChild(svg);
	syncBtn.addEventListener("click", handleSyncDevices);

	header.append(title, count, syncBtn);
	card.appendChild(header);

	const body = document.createElement("div");
	body.className = "card-body";
	body.style.padding = "12px";
	const searchContainer = document.createElement("div");
	searchContainer.className = "device-search-container";
	searchContainer.style.marginBottom = "12px";
	const input = document.createElement("input");
	input.type = "text";
	input.id = "device-search";
	input.className = "setting-input";
	input.placeholder = "🔍 Search devices...";
	input.style.width = "100%";
	searchContainer.appendChild(input);
	const listContainer = document.createElement("div");
	listContainer.className = "device-list-container";
	listContainer.id = "device-list-container";
	const list = document.createElement("ul");
	list.className = "device-list";
	list.id = "device-list";
	const empty = document.createElement("li");
	empty.className = "device-empty";
	empty.id = "device-empty";
	const emptyIcon = document.createElement("span");
	emptyIcon.className = "empty-icon";
	emptyIcon.textContent = "📲";
	const emptyText = document.createElement("span");
	emptyText.textContent = "No devices connected";
	empty.append(emptyIcon, emptyText);
	list.appendChild(empty);
	listContainer.appendChild(list);
	body.append(searchContainer, listContainer);
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
	const title = document.createElement("div");
	title.className = "card-title";
	const icon = document.createElement("span");
	icon.className = "card-icon";
	icon.textContent = "⚙️";
	title.append(icon, "Settings");
	const collapseIcon = document.createElement("span");
	collapseIcon.className = "collapse-icon";
	collapseIcon.textContent = "▼";
	header.append(title, collapseIcon);
	header.onclick = () => card.classList.toggle("collapsed");
	card.appendChild(header);

	const body = document.createElement("div");
	body.className = "card-body";
	const apiItem = document.createElement("div");
	apiItem.className = "setting-item";
	const apiLabel = document.createElement("label");
	apiLabel.className = "setting-label";
	apiLabel.textContent = "AMOS API URL";
	const apiInput = document.createElement("input");
	apiInput.type = "url";
	apiInput.className = "setting-input";
	apiInput.id = "api-url";
	apiInput.placeholder = "https://amos-api.moo-vpn.online";
	apiItem.append(apiLabel, apiInput);
	const perfItem = document.createElement("div");
	perfItem.className = "setting-item";
	const perfLabel = document.createElement("label");
	perfLabel.className = "setting-label";
	perfLabel.append("High Performance Mode");
	const scrcpyStatus = document.createElement("span");
	scrcpyStatus.className = "setting-hint";
	scrcpyStatus.id = "scrcpy-status";
	perfLabel.appendChild(scrcpyStatus);
	const toggleContainer = document.createElement("div");
	toggleContainer.className = "toggle-container";
	const toggleInput = document.createElement("input");
	toggleInput.type = "checkbox";
	toggleInput.id = "toggle-scrcpy";
	toggleInput.className = "toggle-input";
	const toggleLabel = document.createElement("label");
	toggleLabel.htmlFor = "toggle-scrcpy";
	toggleLabel.className = "toggle-label";
	const toggleText = document.createElement("span");
	toggleText.className = "toggle-text";
	toggleText.id = "toggle-scrcpy-text";
	toggleText.textContent = "Requires scrcpy";
	toggleContainer.append(toggleInput, toggleLabel, toggleText);
	perfItem.append(perfLabel, toggleContainer);
	const openWebBtn = document.createElement("button");
	openWebBtn.className = "btn btn-secondary btn-full";
	openWebBtn.id = "btn-open-web";
	openWebBtn.textContent = "🌐 Open AMOS Web UI";

	// Mirror Relay Section
	const relayItem = document.createElement("div");
	relayItem.className = "setting-item";
	const relayLabel = document.createElement("label");
	relayLabel.className = "setting-label";
	relayLabel.textContent = "Browser Mirror (Relay)";
	const relayStatus = document.createElement("span");
	relayStatus.className = "setting-hint";
	relayStatus.id = "relay-status";
	relayStatus.textContent = "Disconnected";
	relayLabel.appendChild(relayStatus);
	const relayToggleContainer = document.createElement("div");
	relayToggleContainer.className = "toggle-container";
	const relayToggleInput = document.createElement("input");
	relayToggleInput.type = "checkbox";
	relayToggleInput.id = "toggle-relay";
	relayToggleInput.className = "toggle-input";
	const relayToggleLabel = document.createElement("label");
	relayToggleLabel.htmlFor = "toggle-relay";
	relayToggleLabel.className = "toggle-label";
	const relayToggleText = document.createElement("span");
	relayToggleText.className = "toggle-text";
	relayToggleText.id = "toggle-relay-text";
	relayToggleText.textContent = "Enable for sharing";
	relayToggleContainer.append(
		relayToggleInput,
		relayToggleLabel,
		relayToggleText,
	);
	relayItem.append(relayLabel, relayToggleContainer);

	body.append(apiItem, perfItem, relayItem, openWebBtn);
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
	const title = document.createElement("div");
	title.className = "mirror-title";
	const titleIcon = document.createElement("span");
	titleIcon.textContent = "📺";
	const deviceName = document.createElement("span");
	deviceName.className = "device-name";
	deviceName.id = "mirror-device-name";
	deviceName.textContent = "Screen Mirror";
	const editName = document.createElement("button");
	editName.className = "mirror-edit-name";
	editName.id = "btn-edit-device-name";
	editName.title = "Edit name";
	editName.textContent = "✎";
	const modeBadge = document.createElement("span");
	modeBadge.className = "mirror-mode-badge mode-connecting";
	modeBadge.id = "mirror-mode-badge";
	modeBadge.textContent = "connecting";
	modeBadge.title = "Connecting…";
	title.append(titleIcon, deviceName, editName, modeBadge);
	const close = document.createElement("button");
	close.className = "mirror-close";
	close.id = "btn-close-mirror";
	close.title = "Close mirror";
	close.textContent = "✕";
	header.append(title, close);
	card.appendChild(header);

	const screenContainer = document.createElement("div");
	screenContainer.className = "mirror-screen-container";
	screenContainer.id = "mirror-screen-container";
	const placeholder = document.createElement("div");
	placeholder.className = "mirror-screen-placeholder";
	placeholder.id = "mirror-placeholder";
	const placeholderIcon = document.createElement("span");
	placeholderIcon.className = "icon";
	placeholderIcon.textContent = "📱";
	const placeholderText = document.createElement("span");
	placeholderText.className = "text";
	placeholderText.textContent = "Select a device to start mirroring";
	placeholder.append(placeholderIcon, placeholderText);
	const loading = document.createElement("div");
	loading.className = "mirror-loading";
	loading.id = "mirror-loading";
	loading.style.display = "none";
	const spinner = document.createElement("div");
	spinner.className = "spinner";
	const loadingText = document.createElement("span");
	loadingText.textContent = "Connecting to device...";
	loading.append(spinner, loadingText);
	const mirrorScreen = document.createElement("img");
	mirrorScreen.className = "mirror-screen";
	mirrorScreen.id = "mirror-screen";
	mirrorScreen.style.display = "none";
	mirrorScreen.alt = "Device Screen";
	// Canvas for video classes (scrcpy, ADB video) - renders directly for GPU-accelerated display
	const mirrorCanvas = document.createElement("canvas");
	mirrorCanvas.className = "mirror-screen-canvas";
	mirrorCanvas.id = "mirror-screen-canvas";
	mirrorCanvas.style.display = "none";
	const secureNotice = document.createElement("div");
	secureNotice.className = "mirror-secure-notice";
	secureNotice.id = "mirror-secure-notice";
	secureNotice.style.display = "none";
	secureNotice.textContent =
		"This secure screen may appear blank. Enter PIN directly on device.";
	screenContainer.append(
		placeholder,
		loading,
		mirrorScreen,
		mirrorCanvas,
		secureNotice,
	);
	card.appendChild(screenContainer);

	const controls = document.createElement("div");
	controls.className = "mirror-controls";
	controls.id = "mirror-controls";
	controls.style.display = "none";
	[
		["btn-mirror-back", "Back", "⬅"],
		["btn-mirror-home", "Home", "🏠"],
		["btn-mirror-enter", "Enter", "↵"],
		["btn-mirror-power", "Power", "⏻"],
	].forEach(([id, titleText, text]) => {
		const button = document.createElement("button");
		button.className = "btn btn-secondary";
		button.id = id;
		button.title = titleText;
		button.textContent = text;
		controls.appendChild(button);
	});
	card.appendChild(controls);

	const status = document.createElement("div");
	status.className = "mirror-status";
	status.id = "mirror-status";
	[
		["●", "mirror-battery"],
		["📶", "mirror-wifi"],
	].forEach(([iconText, id]) => {
		const item = document.createElement("div");
		item.className = "status-item";
		const icon = document.createElement("span");
		icon.textContent = iconText;
		const value = document.createElement("span");
		value.id = id;
		value.textContent = "—";
		item.append(icon, value);
		status.appendChild(item);
	});
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
	const title = document.createElement("div");
	title.className = "card-title";
	const icon = document.createElement("span");
	icon.className = "card-icon";
	icon.textContent = "📋";
	title.append(icon, "Activity Logs");
	const controls = document.createElement("div");
	controls.className = "log-controls";
	const filter = document.createElement("select");
	filter.id = "log-filter";
	filter.className = "log-filter-select";
	filter.style.background = "var(--bg-tertiary)";
	filter.style.border = "1px solid var(--border-color)";
	filter.style.color = "var(--text-primary)";
	filter.style.padding = "4px 8px";
	filter.style.borderRadius = "6px";
	filter.style.fontSize = "11px";
	[
		["all", "All"],
		["info", "Info"],
		["warn", "Warning"],
		["error", "Error"],
	].forEach(([value, label]) => {
		const option = document.createElement("option");
		option.value = value;
		option.textContent = label;
		filter.appendChild(option);
	});
	const exportBtn = document.createElement("button");
	exportBtn.className = "btn btn-small btn-ghost";
	exportBtn.id = "btn-export-logs";
	exportBtn.title = "Export logs";
	exportBtn.textContent = "📤";
	const clearBtn = document.createElement("button");
	clearBtn.className = "btn btn-small btn-ghost";
	clearBtn.id = "btn-clear-logs";
	clearBtn.title = "Clear logs";
	clearBtn.textContent = "🗑️";
	controls.append(filter, exportBtn, clearBtn);
	header.append(title, controls);
	card.appendChild(header);

	const body = document.createElement("div");
	body.className = "log-container";
	body.id = "log-container";
	body.style.flex = "1";
	const content = document.createElement("div");
	content.className = "log-content";
	content.id = "log-content";
	body.appendChild(content);
	card.appendChild(body);

	return card;
}

function buildFooter(): HTMLElement {
	const footer = document.createElement("footer");
	footer.className = "app-footer";
	const title = document.createElement("span");
	title.textContent = "AMOS Device Management";
	const sep = document.createElement("span");
	sep.className = "footer-sep";
	sep.textContent = "•";
	const year = document.createElement("span");
	year.textContent = String(new Date().getFullYear());
	footer.append(title, sep, year);
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

	// If scrcpy is enabled, start scrcpy for this device
	if (scrcpyEnabled) {
		try {
			const result = await invoke<ScrcpyLaunchResult>("start_scrcpy", {
				serial: device.serial,
			});
			logScrcpyResult(result);
		} catch (err) {
			addLog("error", `Failed to start scrcpy: ${err}`);
		}
		return;
	}

	// Otherwise, start built-in mirroring
	await startMirror(device);
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
let refreshFrameCount = 0; // Counter for black screen detection
let consecutiveBlackFrames = 0; // Track sustained black to avoid false positives
const MAX_SCREENSHOT_ERRORS = 3;

async function _refreshScreenshot(): Promise<void> {
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

// Swipe gesture state
let pointerStartX = 0;
let pointerStartY = 0;
let isPointerDown = false;
let currentPointerId: number | null = null;
let lastPointerX = 0;
let lastPointerY = 0;
const SWIPE_THRESHOLD = 30; // pixels of movement to distinguish swipe from tap
const SWIPE_DURATION_MS = 250; // Android input swipe duration

/**
 * Calculate screen coordinates from pointer event, accounting for object-fit: contain
 * Returns null if click is in letterbox area
 */
function getScreenCoords(event: PointerEvent): { x: number; y: number } | null {
	// Use the video <canvas> when it is visible (scrcpy / ADB video); otherwise
	// fall back to the screenshot <img>. Both are children of
	// #mirror-screen-container, so the container rect is the same for either.
	const container = document.getElementById("mirror-screen-container");
	if (!container) return null;
	const canvasEl = document.getElementById(
		"mirror-screen-canvas",
	) as HTMLCanvasElement | null;
	const imgEl = document.getElementById(
		"mirror-screen",
	) as HTMLImageElement | null;
	let imgWidth: number;
	let imgHeight: number;
	if (canvasEl && canvasEl.style.display !== "none" && canvasEl.width) {
		imgWidth = canvasEl.width;
		imgHeight = canvasEl.height;
	} else if (imgEl && imgEl.naturalWidth) {
		imgWidth = imgEl.naturalWidth;
		imgHeight = imgEl.naturalHeight;
	} else {
		return null;
	}

	const containerRect = container.getBoundingClientRect();
	const containerWidth = containerRect.width;
	const containerHeight = containerRect.height;

	// Calculate scale and offsets for object-fit: contain
	const scaleToFit = Math.min(
		containerWidth / imgWidth,
		containerHeight / imgHeight,
	);
	const displayedWidth = imgWidth * scaleToFit;
	const displayedHeight = imgHeight * scaleToFit;

	// Calculate the letterbox/pillarbox offsets (centered)
	const offsetX = (containerWidth - displayedWidth) / 2;
	const offsetY = (containerHeight - displayedHeight) / 2;

	// Check if click is within the visible image area
	const clickX = event.clientX - containerRect.left;
	const clickY = event.clientY - containerRect.top;

	if (
		clickX < offsetX ||
		clickX > offsetX + displayedWidth ||
		clickY < offsetY ||
		clickY > offsetY + displayedHeight
	) {
		// Click is in letterbox area, ignore
		return null;
	}

	// Convert click coordinates to image coordinates
	const imageX = (clickX - offsetX) / scaleToFit;
	const imageY = (clickY - offsetY) / scaleToFit;

	return { x: Math.round(imageX), y: Math.round(imageY) };
}

function handlePointerDown(event: PointerEvent): void {
	// Prevent browser gestures
	event.preventDefault();

	const coords = getScreenCoords(event);
	if (!coords) return;

	isPointerDown = true;
	pointerStartX = coords.x;
	pointerStartY = coords.y;
	lastPointerX = coords.x;
	lastPointerY = coords.y;
	currentPointerId = event.pointerId;

	// Capture pointer so we receive pointerup even if cursor leaves element
	(event.target as HTMLElement).setPointerCapture(event.pointerId);
}

function handlePointerMove(event: PointerEvent): void {
	if (!isPointerDown || event.pointerId !== currentPointerId) return;

	const coords = getScreenCoords(event);
	if (!coords) return;

	lastPointerX = coords.x;
	lastPointerY = coords.y;
}

function handlePointerUp(event: PointerEvent): void {
	if (event.pointerId !== currentPointerId) return;

	if (!isPointerDown) {
		isPointerDown = false;
		currentPointerId = null;
		return;
	}

	isPointerDown = false;
	currentPointerId = null;

	const coords = getScreenCoords(event);
	const endX = coords?.x ?? lastPointerX;
	const endY = coords?.y ?? lastPointerY;

	const deltaX = endX - pointerStartX;
	const deltaY = endY - pointerStartY;
	const distance = Math.sqrt(deltaX * deltaX + deltaY * deltaY);

	if (distance < SWIPE_THRESHOLD) {
		// This is a tap
		invoke("device_tap", {
			serial: currentMirroringDevice,
			x: pointerStartX,
			y: pointerStartY,
		})
			.then(() => addLog("debug", `Tap: ${pointerStartX},${pointerStartY}`))
			.catch((err) => addLog("error", `Tap failed: ${err}`));
	} else {
		// This is a swipe
		invoke("device_swipe", {
			serial: currentMirroringDevice,
			x1: pointerStartX,
			y1: pointerStartY,
			x2: endX,
			y2: endY,
			duration_ms: SWIPE_DURATION_MS,
		})
			.then(() =>
				addLog(
					"debug",
					`Swipe: ${pointerStartX},${pointerStartY} -> ${endX},${endY}`,
				),
			)
			.catch((err) => addLog("error", `Swipe failed: ${err}`));
	}
}

function handlePointerCancel(event: PointerEvent): void {
	if (event.pointerId !== currentPointerId) return;

	// Emit swipe with last known position if we had movement
	if (
		isPointerDown &&
		(pointerStartX !== lastPointerX || pointerStartY !== lastPointerY)
	) {
		invoke("device_swipe", {
			serial: currentMirroringDevice,
			x1: pointerStartX,
			y1: pointerStartY,
			x2: lastPointerX,
			y2: lastPointerY,
			duration_ms: SWIPE_DURATION_MS,
		})
			.then(() =>
				addLog(
					"debug",
					`Swipe: ${pointerStartX},${pointerStartY} -> ${lastPointerX},${lastPointerY}`,
				),
			)
			.catch((err) => addLog("error", `Swipe failed: ${err}`));
	}

	isPointerDown = false;
	currentPointerId = null;
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

/**
 * Log scrcpy launch result truthfully based on backend result.
 * Distinguishes:
 * - Success + focus succeeded
 * - Success + focus failed
 * - Launch failed
 */
function logScrcpyResult(result: ScrcpyLaunchResult): void {
	if (!result.success) {
		addLog("error", `scrcpy failed: ${result.message}`);
		return;
	}

	// Success
	const pidStr = result.pid ? ` (PID: ${result.pid})` : "";
	addLog("info", `scrcpy launched${pidStr} - window: "${result.window_title}"`);

	if (result.focus_attempted && result.focus_succeeded === true) {
		addLog("info", `scrcpy window focused to front`);
	} else if (result.focus_attempted && result.focus_succeeded === false) {
		addLog(
			"warn",
			`scrcpy window NOT auto-focused. Look in Dock, Mission Control, or other Spaces.`,
		);
		addLog(
			"info",
			`To enable focus on macOS: System Settings → Privacy & Security → Accessibility → enable AMOS Companion`,
		);
	} else {
		addLog(
			"info",
			`If you can't see the window, check Dock or Mission Control.`,
		);
	}
}

async function handleScrcpyToggle(): Promise<void> {
	const toggle = document.getElementById("toggle-scrcpy") as HTMLInputElement;
	if (!toggle) return;

	if (toggle.checked) {
		if (!scrcpyAvailable) {
			addLog(
				"warn",
				"scrcpy-server not found. Install scrcpy-server.jar to enable high-performance mirroring.",
			);
			toggle.checked = false;
			scrcpyEnabled = false;
			return;
		}
		// Enable scrcpy-server mode for mirror
		scrcpyEnabled = true;
		addLog("info", "[SCRCPY] scrcpy-server mode enabled - will use for mirror");

		// If mirror is currently running, restart it with scrcpy-server
		if (currentMirroringDevice) {
			addLog("info", `[MIRROR] Restarting mirror with scrcpy-server mode...`);
			const device: DeviceInfo = selectedDevice || {
				serial: currentMirroringDevice,
				model: "",
				status: "device",
				resolution: null,
				battery: null,
			};
			await stopMirrorInternal();
			await startMirror(device);
		}
	} else {
		// Disable scrcpy-server mode
		scrcpyEnabled = false;
		addLog(
			"info",
			"[SCRCPY] scrcpy-server mode disabled - using ADB screenrecord",
		);

		// If mirror is currently running with scrcpy-server, restart with ADB
		if (currentMirroringDevice && scrcpyServerStream) {
			addLog("info", `[MIRROR] Restarting mirror with ADB mode...`);
			const device: DeviceInfo = selectedDevice || {
				serial: currentMirroringDevice,
				model: "",
				status: "device",
				resolution: null,
				battery: null,
			};
			await stopMirrorInternal();
			await startMirror(device);
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

// ─── Device Sync ─────────────────────────────────────────────────────────────

async function handleSyncDevices(): Promise<void> {
	const syncBtn = document.getElementById(
		"btn-sync-devices",
	) as HTMLButtonElement;
	if (syncBtn) {
		syncBtn.disabled = true;
		syncBtn.textContent = "⏳";
		syncBtn.classList.add("syncing");
	}

	addLog("info", "Syncing devices...");

	try {
		// Refresh status to trigger device-agent heartbeat
		const status = await invoke<AgentStatus>("get_status");
		state = status;
		display = computeDisplay();
		refreshUI();

		// Get device list
		const deviceList = await invoke<DeviceList>("get_devices");
		refreshDeviceList(deviceList.devices);
		addLog("info", `Found ${deviceList.devices.length} device(s)`);
	} catch (err) {
		addLog("error", `Sync failed: ${err}`);
	}

	if (syncBtn) {
		syncBtn.disabled = false;
		syncBtn.textContent = "🔄";
		syncBtn.classList.remove("syncing");
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
	const headerVersion = document.getElementById("header-version");
	if (headerVersion) {
		headerVersion.textContent = getCompanionVersionLabel();
	}

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
		const startIcon = document.createElement("span");
		startIcon.className = "btn-icon";
		if (display === "running") {
			btnStart.disabled = true;
			startIcon.textContent = "✓";
			btnStart.replaceChildren(startIcon, document.createTextNode("Running"));
			btnStop.disabled = false;
		} else {
			btnStart.disabled = false;
			startIcon.textContent = "▶";
			btnStart.replaceChildren(
				startIcon,
				document.createTextNode("Start Agent"),
			);
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
			// ADB reports connected devices as status "device" (not "connected")
			const isOnline =
				device.status === "device" || device.status === "connected";
			li.className = `device-item ${isOnline ? "device-online" : "device-offline"}`;
			const isMirroring = currentMirroringDevice === device.serial;
			const icon = document.createElement("span");
			icon.className = `device-icon ${isOnline ? "online" : "offline"}`;
			const info = document.createElement("div");
			info.className = "device-info";
			const name = document.createElement("span");
			name.className = "device-name";
			name.textContent = device.model || "Unknown Device";
			const serial = document.createElement("span");
			serial.className = "device-serial";
			serial.textContent = device.serial;
			info.append(name, serial);
			const actions = document.createElement("div");
			actions.className = "device-actions";
			const button = document.createElement("button");
			button.className = `btn btn-small ${isMirroring ? "btn-primary" : "btn-secondary"} btn-mirror`;
			button.title = isMirroring ? "Stop mirroring" : "Start mirroring";
			button.dataset.device = device.serial;
			button.textContent = isMirroring ? "■" : "👁️";
			actions.appendChild(button);
			li.append(icon, info, actions);
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

// WebCodecs-based video player using Tauri events (no WebSocket needed).
// ADB Video Streaming (Built-in Mirror Mode)
//
// ✅ STATUS: WORKING
// This is the built-in mirror mode using adb screenrecord + WebCodecs.
//
// PERFORMANCE: Limited due to ADB overhead (~100-300ms latency per frame).
// For better performance, use scrcpy mode instead (scrcpy_server.rs).
//
// Features:
// 1. Waits for actual first frame to render before declaring success
// 2. Automatic fallback to screenshot polling if video doesn't render
// 3. 5-second timeout prevents false success states
// 4. Backend now emits structured payloads `{ bytes, key }`. The key flag is
//    set by the backend while grouping NALs into access units (it tracks IDR
//    presence). The frontend just trusts the flag — no more first-NAL-byte
//    guessing in TypeScript.

class VideoStream {
	private unlistenFrame: (() => void) | null = null;
	private decoder: globalThis.VideoDecoder | null = null;
	private canvas: HTMLCanvasElement | null = null;
	private ctx: CanvasRenderingContext2D | null = null;
	private running = false;
	private _stopped = false;
	private width = 1080;
	private height = 1920;
	private _firstFrameRendered = false;
	private _frameRenderResolve: ((value: boolean) => void) | null = null;
	private _firstFrameTimeout: ReturnType<typeof setTimeout> | null = null;
	/** Timeout in ms to wait for first frame before giving up */
	private static readonly FIRST_FRAME_TIMEOUT_MS = 5000;
	/**
	 * FIX 1: bounded startup queue. If a frame event arrives from the backend
	 * BEFORE canvas/ctx/decoder are wired up, the frame is queued instead of
	 * being dropped. Once canvas+ctx are created in start(), the queue is
	 * drained in order. Bounded so a stall on the backend can't blow up
	 * memory.
	 */
	private _earlyFrames: Array<{ bytes: ArrayBuffer; isKey: boolean }> = [];
	private static readonly EARLY_FRAMES_CAP = 16;

	constructor(private serial: string) {}

	/** Returns true once the first frame has been successfully rendered */
	get firstFrameRendered(): boolean {
		return this._firstFrameRendered;
	}

	/**
	 * Start the video stream and wait for the first frame to render.
	 * Returns a promise that resolves to true if a frame rendered within the timeout,
	 * or false if the stream failed or timed out.
	 */
	async start(): Promise<boolean> {
		// Start backend stream via Tauri command
		try {
			// CRITICAL: Subscribe to frames BEFORE calling backend
			// to avoid missing initial SPS/PPS/IDR config packets
			await this.subscribeToFrames();

			const info = await invoke<{
				width: number;
				height: number;
				running: boolean;
				event_name: string;
			}>("start_video_stream", { serial: this.serial });

			if (!info.running) {
				throw new Error("Stream failed to start");
			}

			this.width = info.width;
			this.height = info.height;

			// Create canvas for rendering (use shared canvas for video classes)
			const sharedCanvas = document.getElementById(
				"mirror-screen-canvas",
			) as HTMLCanvasElement;
			this.canvas = sharedCanvas || document.createElement("canvas");
			this.canvas.width = this.width;
			this.canvas.height = this.height;
			this.ctx = this.canvas.getContext("2d");
			// Show canvas, hide img
			const mirrorScreen = document.getElementById(
				"mirror-screen",
			) as HTMLImageElement;
			if (mirrorScreen) mirrorScreen.style.display = "none";
			if (this.canvas) this.canvas.style.display = "block";

			this.running = true;
			addLog(
				"info",
				`Video stream started (${this.width}x${this.height}) via Tauri events`,
			);

			// FIX 1: drain any frames that arrived between subscribe and canvas
			// creation. Decoder init is lazy and happens on first decode call.
			this.drainEarlyFrames();

			// Wait for first frame with timeout - this is the critical fix
			// We don't declare success until we actually have a rendered frame
			return this.waitForFirstFrame();
		} catch (error) {
			addLog("error", `Failed to start video stream: ${error}`);
			return false;
		}
	}

	/**
	 * Wait for the first frame to be rendered, with a timeout.
	 * Returns true if first frame rendered within timeout, false otherwise.
	 */
	private waitForFirstFrame(): Promise<boolean> {
		return new Promise((resolve) => {
			this._frameRenderResolve = resolve;

			// Set timeout for first frame
			this._firstFrameTimeout = setTimeout(() => {
				if (!this._firstFrameRendered) {
					addLog(
						"warn",
						`Video stream: No frame rendered within ${VideoStream.FIRST_FRAME_TIMEOUT_MS}ms, will fall back to screenshots`,
					);
					// Resolve false - video stream isn't working, fall back to screenshots
					this._frameRenderResolve?.(false);
					this._frameRenderResolve = null;
				}
			}, VideoStream.FIRST_FRAME_TIMEOUT_MS);
		});
	}

	private async subscribeToFrames(): Promise<void> {
		// Unsubscribe from any previous subscription
		if (this.unlistenFrame) {
			this.unlistenFrame();
			this.unlistenFrame = null;
		}

		const { listen } = await import("@tauri-apps/api/event");
		// New structured payload: { bytes: number[], key: boolean }
		// The backend has already grouped NALs into access units and tells us
		// whether the AU contains an IDR (keyframe) — no more guessing from
		// the first NAL byte on the frontend.
		const unlisten = await listen<{ bytes: number[]; key: boolean }>(
			"video-frame",
			(event) => {
				if (this._stopped) return;
				const data = new Uint8Array(event.payload.bytes).buffer;
				this.handleFrame(data, event.payload.key);
			},
		);
		this.unlistenFrame = unlisten;
	}

	/**
	 * FIX 1: if a frame arrives before canvas/ctx are ready, queue it in
	 * order. Otherwise dispatch for decoding. Decoder creation is lazy inside
	 * decodeWithWebCodecs(), so we must NOT queue merely because decoder is
	 * still null, or the first AU can get stuck forever after drainEarlyFrames()
	 * already ran.
	 */
	private handleFrame(data: ArrayBuffer, isKey: boolean): void {
		if (this._stopped) return;
		// If render objects are not yet ready, queue. The frame is dropped if
		// the queue is already at the cap (the cap is small so we never fall
		// far behind; this is a startup-only buffer).
		if (!this.canvas || !this.ctx) {
			if (this._earlyFrames.length >= VideoStream.EARLY_FRAMES_CAP) {
				// Drop the oldest queued frame to keep the queue bounded.
				this._earlyFrames.shift();
			}
			this._earlyFrames.push({ bytes: data, isKey });
			return;
		}

		// Try WebCodecs first (Chrome 94+)
		if (typeof VideoDecoder !== "undefined") {
			this.decodeWithWebCodecs(data, isKey);
		} else {
			// Fallback: try to display as PNG if possible
			this.displayAsImage(data);
		}
	}

	/**
	 * FIX 1: drain queued early frames in order. Called from start() after
	 * canvas+ctx are created. Each skipped frame is consistently dropped
	 * after the timeout fires, not replayed into a stopped stream.
	 */
	private drainEarlyFrames(): void {
		if (this._earlyFrames.length === 0) return;
		// Move the queue out so any synchronous handler that re-enters cannot
		// see/append during iteration.
		const queued = this._earlyFrames;
		this._earlyFrames = [];
		for (const f of queued) {
			if (this._stopped) return;
			if (typeof VideoDecoder !== "undefined") {
				this.decodeWithWebCodecs(f.bytes, f.isKey);
			} else {
				this.displayAsImage(f.bytes);
			}
		}
	}

	private async decodeWithWebCodecs(
		data: ArrayBuffer,
		isKey: boolean,
	): Promise<void> {
		if (!this.decoder) {
			this.initDecoder();
		}

		if (!this.decoder || !this.canvas || !this.ctx) return;

		try {
			const chunk = new EncodedVideoChunk({
				// Backend tells us the truth: this AU was tagged `key` iff it
				// contained an IDR slice. Use the flag directly instead of
				// inspecting NAL bytes ourselves.
				type: isKey ? "key" : "delta",
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
					// Scale and draw frame to canvas (canvas is now visible)
					this.ctx.drawImage(
						frame,
						0,
						0,
						this.canvas.width,
						this.canvas.height,
					);
					frame.close();

					// Mark first frame as rendered - this is the success signal
					if (!this._firstFrameRendered) {
						this._firstFrameRendered = true;
						addLog("info", "First video frame rendered successfully");

						// Clear timeout and resolve the wait promise
						if (this._firstFrameTimeout) {
							clearTimeout(this._firstFrameTimeout);
							this._firstFrameTimeout = null;
						}
						if (this._frameRenderResolve) {
							this._frameRenderResolve(true);
							this._frameRenderResolve = null;
						}
					}
				}
			},
			error: (error) => {
				addLog("warn", `Decoder error: ${error}`);
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
				}
				URL.revokeObjectURL(url);
			};
			img.src = url;
		} catch {
			// Silently ignore
		}
	}

	stop(): void {
		this.running = false;
		// Set _stopped so any frame handler that was already scheduled
		// after a previous continue can short-circuit and not render/stop.
		this._stopped = true;
		// Drop any queued startup frames so they don't get replayed after
		// a future restart of the same stream instance.
		this._earlyFrames = [];

		// Hide canvas, restore img for screenshot path
		if (this.canvas) this.canvas.style.display = "none";
		const mirrorScreen = document.getElementById(
			"mirror-screen",
		) as HTMLImageElement;
		if (mirrorScreen) mirrorScreen.style.display = "block";

		// Unsubscribe from Tauri event
		if (this.unlistenFrame) {
			this.unlistenFrame();
			this.unlistenFrame = null;
		}

		if (this.decoder) {
			this.decoder.close();
			this.decoder = null;
		}

		// Stop Rust backend stream
		invoke("stop_video_stream").catch(() => {});
	}
}

/**
 * ✅ WORKING: Scrcpy-Server Video Streaming
 *
 * Streams video from scrcpy-server via Tauri events into #mirror-screen div.
 * This is the higher-performance alternative to ADB screenrecord.
 *
 * Features:
 * 1. scrcpy-server streams H.264 via ADB port forwarding
 * 2. Backend reads frames and emits via Tauri events
 * 3. Frontend receives events and decodes with WebCodecs
 * 4. Renders directly in #mirror-screen div (same as ADB mode)
 */
class ScrcpyVideoStream {
	private unlistenFrame: (() => void) | null = null;
	private unlistenStarted: (() => void) | null = null;
	private decoder: globalThis.VideoDecoder | null = null;
	private canvas: HTMLCanvasElement | null = null;
	private ctx: CanvasRenderingContext2D | null = null;
	private running = false;
	private _stopped = false;
	private width = 1080;
	private height = 1920;
	private _firstFrameRendered = false;
	private _frameRenderResolve: ((value: boolean) => void) | null = null;
	private _firstFrameTimeout: ReturnType<typeof setTimeout> | null = null;
	private static readonly FIRST_FRAME_TIMEOUT_MS = 5000;

	constructor(private serial: string) {}

	get firstFrameRendered(): boolean {
		return this._firstFrameRendered;
	}

	/**
	 * Start scrcpy-server streaming.
	 * Returns true if first frame rendered within timeout, false otherwise.
	 */
	async start(): Promise<boolean> {
		try {
			addLog(
				"info",
				`[SCRCPY] Starting scrcpy-server mirror for ${this.serial}...`,
			);

			// CRITICAL: Subscribe to events BEFORE calling backend
			// to avoid missing initial packets/frames
			await this.subscribeToEvents();

			const info = await invoke<{
				width: number;
				height: number;
				running: boolean;
				event_name: string;
			}>("start_scrcpy_mirror", { serial: this.serial });

			// FIX Issue 3: Only proceed if the backend genuinely reports the
			// scrcpy-server process is alive on the device. Previously we
			// accepted width/height defaults of (1920,1920) and printed
			// "Server started" even when the server process was already dead.
			if (!info.running) {
				addLog(
					"warn",
					"[SCRCPY] scrcpy-server process NOT alive on device, will fall back to ADB screenrecord",
				);
				this.stop();
				return false;
			}

			this.width = info.width || 1080;
			this.height = info.height || 1920;

			// Create canvas for rendering (use shared canvas for video classes)
			const sharedCanvas = document.getElementById(
				"mirror-screen-canvas",
			) as HTMLCanvasElement;
			this.canvas = sharedCanvas || document.createElement("canvas");
			this.canvas.width = this.width;
			this.canvas.height = this.height;
			this.ctx = this.canvas.getContext("2d");
			// Show canvas, hide img
			const mirrorScreen = document.getElementById(
				"mirror-screen",
			) as HTMLImageElement;
			if (mirrorScreen) mirrorScreen.style.display = "none";
			if (this.canvas) this.canvas.style.display = "block";

			this.running = true;

			// Don't log "Server started" until we know a real frame is rendered.
			// The previous wording claimed success before any pixel was on screen.
			addLog(
				"info",
				`[SCRCPY] scrcpy-server alive (${this.width}x${this.height}), waiting for first frame...`,
			);

			return this.waitForFirstFrame();
		} catch (error) {
			addLog("error", `[SCRCPY] Failed to start: ${error}`);
			return false;
		}
	}

	private async subscribeToEvents(): Promise<void> {
		const { listen } = await import("@tauri-apps/api/event");

		// Listen for debug messages from backend
		await listen<string>("scrcpy-debug", (event) => {
			addLog("info", `[SCRCPY] ${event.payload}`);
		});

		// Listen for stream started event with dimensions
		this.unlistenStarted = await listen<{
			serial: string;
			port: number;
			width?: number;
			height?: number;
			message?: string;
		}>("scrcpy-stream-started", (event) => {
			if (event.payload.message) {
				addLog("info", `[SCRCPY] ${event.payload.message}`);
			}
			if (event.payload.width && event.payload.height) {
				this.width = event.payload.width;
				this.height = event.payload.height;
				addLog("info", `[SCRCPY] Device info: ${this.width}x${this.height}`);
			}
		});

		// Listen for video frames
		this.unlistenFrame = await listen<{ bytes: number[]; key: boolean }>(
			"scrcpy-frame",
			(event) => {
				if (this._stopped) return;
				const data = new Uint8Array(event.payload.bytes).buffer;
				this.handleFrame(data, event.payload.key);
			},
		);
	}

	private handleFrame(data: ArrayBuffer, isKey: boolean): void {
		if (typeof VideoDecoder !== "undefined") {
			this.decodeWithWebCodecs(data, isKey);
		} else {
			addLog("warn", "[SCRCPY] WebCodecs not available");
		}
	}

	private decodeWithWebCodecs(data: ArrayBuffer, isKey: boolean): void {
		if (!this.decoder) {
			addLog("info", `[SCRCPY] Initializing decoder...`);
			this.initDecoder();
		}

		if (!this.decoder || !this.canvas || !this.ctx) {
			addLog("warn", `[SCRCPY] Decoder not ready, dropping frame`);
			return;
		}

		try {
			const chunk = new EncodedVideoChunk({
				type: isKey ? "key" : "delta",
				timestamp: Date.now() * 1000,
				data,
			});

			this.decoder.decode(chunk);
		} catch (error) {
			addLog("warn", `[SCRCPY] Decode error: ${error}`);
		}
	}

	private initDecoder(): void {
		if (typeof VideoDecoder === "undefined") return;

		this.decoder = new VideoDecoder({
			output: (frame) => {
				if (this.canvas && this.ctx) {
					this.ctx.drawImage(
						frame,
						0,
						0,
						this.canvas.width,
						this.canvas.height,
					);
					frame.close();

					// Mark first frame
					if (!this._firstFrameRendered) {
						this._firstFrameRendered = true;
						addLog("info", "[SCRCPY] First frame rendered successfully!");
						setMirrorModeBadge("scrcpy");

						if (this._firstFrameTimeout) {
							clearTimeout(this._firstFrameTimeout);
							this._firstFrameTimeout = null;
						}
						if (this._frameRenderResolve) {
							this._frameRenderResolve(true);
							this._frameRenderResolve = null;
						}
					}
				}
			},
			error: (error) => {
				addLog("warn", `[SCRCPY] Decoder error: ${error}`);
			},
		});

		this.decoder.configure({
			codec: "avc1.64001f",
			codedWidth: this.width,
			codedHeight: this.height,
		});
	}

	private waitForFirstFrame(): Promise<boolean> {
		return new Promise((resolve) => {
			this._frameRenderResolve = resolve;

			this._firstFrameTimeout = setTimeout(() => {
				if (!this._firstFrameRendered) {
					addLog(
						"warn",
						"[SCRCPY] No frame rendered within timeout, stopping...",
					);
					this._frameRenderResolve?.(false);
					this._frameRenderResolve = null;
				}
			}, ScrcpyVideoStream.FIRST_FRAME_TIMEOUT_MS);
		});
	}

	stop(): void {
		this.running = false;
		this._stopped = true;

		if (this._firstFrameTimeout) {
			clearTimeout(this._firstFrameTimeout);
			this._firstFrameTimeout = null;
		}

		if (this._frameRenderResolve) {
			this._frameRenderResolve(false);
			this._frameRenderResolve = null;
		}

		// Hide canvas, restore img for screenshot path
		if (this.canvas) this.canvas.style.display = "none";
		const mirrorScreen = document.getElementById(
			"mirror-screen",
		) as HTMLImageElement;
		if (mirrorScreen) mirrorScreen.style.display = "block";

		if (this.unlistenFrame) {
			this.unlistenFrame();
			this.unlistenFrame = null;
		}

		if (this.unlistenStarted) {
			this.unlistenStarted();
			this.unlistenStarted = null;
		}

		if (this.decoder) {
			this.decoder.close();
			this.decoder = null;
		}

		invoke("stop_scrcpy_mirror").catch(() => {});
	}
}

type MirrorMode = "connecting" | "scrcpy" | "adb" | "secure" | "disconnected";

const MIRROR_MODE_META: Record<
	MirrorMode,
	{ cls: string; label: string; tip: string }
> = {
	connecting: {
		cls: "mode-connecting",
		label: "connecting",
		tip: "Starting mirror…",
	},
	scrcpy: {
		cls: "mode-scrcpy",
		label: "scrcpy",
		tip: "Live video via scrcpy (high performance)",
	},
	adb: {
		cls: "mode-adb",
		label: "ADB",
		tip: "Screenshot polling via ADB (slower; may black out on secure screens)",
	},
	secure: {
		cls: "mode-secure",
		label: "secure",
		tip: "Secure screen — content hidden. Enter PIN on the device.",
	},
	disconnected: {
		cls: "mode-disconnected",
		label: "off",
		tip: "Mirror stopped",
	},
};

function setMirrorModeBadge(mode: MirrorMode): void {
	const badge = document.getElementById("mirror-mode-badge");
	const meta = MIRROR_MODE_META[mode];
	if (badge) {
		badge.className = `mirror-mode-badge ${meta.cls}`;
		badge.textContent = meta.label;
		badge.title = meta.tip;
	}
	// Keep the toggle hint in sync: "✓ Active" only when scrcpy is really running.
	const toggleStatus = document.getElementById("scrcpy-status");
	if (toggleStatus) {
		toggleStatus.textContent = mode === "scrcpy" ? "✓ Active" : "✓ Available";
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
	setMirrorModeBadge("connecting");

	addLog("info", `Starting mirror for ${device.model || device.serial}...`);

	// Always clear existing polling and video stream first (even for same device)
	// This prevents interval stacking if user clicks same device again
	if (videoStream) {
		videoStream.stop();
		videoStream = null;
	}
	if (screenshotPollingInterval) {
		clearInterval(screenshotPollingInterval);
		screenshotPollingInterval = null;
	}

	// Stop any existing mirror session if it's a different device
	// Don't clear currentMirroringDevice here - we need it for same-device restarts
	const isSameDevice = currentMirroringDevice === device.serial;
	if (currentMirroringDevice && !isSameDevice) {
		addLog("info", `Stopping previous mirror for ${currentMirroringDevice}...`);
		await stopMirrorInternal(); // Internal stop that doesn't show log
	}

	// Show loading state
	if (mirrorPlaceholder) mirrorPlaceholder.style.display = "none";
	if (mirrorLoading) mirrorLoading.style.display = "flex";
	if (mirrorScreen) mirrorScreen.style.display = "none";
	if (mirrorControls) mirrorControls.style.display = "flex";
	if (mirrorDeviceName)
		mirrorDeviceName.textContent = getDeviceDisplayName(device);

	// Set the device AFTER clearing old session
	currentMirroringDevice = device.serial;

	// Try real video streaming first (screenrecord + WebCodecs)
	// Falls back to screenshot polling if video stream fails to start
	addLog("info", `Starting mirror for ${device.serial}...`);
	const videoStarted = await tryStartVideoStream(device.serial);
	if (!videoStarted) {
		addLog(
			"info",
			`Falling back to screenshot polling for ${device.serial}...`,
		);
		setMirrorModeBadge("adb");
		await refreshMirrorScreen(device.serial);

		screenshotPollingInterval = setInterval(async () => {
			if (currentMirroringDevice) {
				await refreshMirrorScreen(currentMirroringDevice);
			}
		}, MIRROR_REFRESH_MS);
	}

	// Only show the <img> when using screenshot polling. When a video stream
	// (scrcpy or ADB) started, it hides the <img> and shows the canvas itself;
	// re-showing the <img> here would display BOTH elements at once (a stale
	// screenshot + the live video, each ~50% of the panel).
	if (!videoStarted && mirrorScreen) mirrorScreen.style.display = "block";
	if (mirrorLoading) mirrorLoading.style.display = "none";
	addLog("info", `Mirror started for ${device.serial}`);

	// Update device info
	if (mirrorBattery)
		mirrorBattery.textContent = device.battery ? `${device.battery}%` : "—";
	if (mirrorWifi) mirrorWifi.textContent = "—";
}

/**
 * Try to start real video streaming.
 *
 * Priority:
 * 1. If scrcpyEnabled and scrcpyAvailable: try scrcpy-server first (higher quality)
 * 2. Fall back to ADB screenrecord (works but slower)
 *
 * Returns true only if a frame was actually rendered.
 * The caller should fall back to screenshot polling if this returns false.
 */
async function tryStartVideoStream(serial: string): Promise<boolean> {
	if (videoStream) {
		videoStream.stop();
		videoStream = null;
	}
	if (scrcpyServerStream) {
		scrcpyServerStream.stop();
		scrcpyServerStream = null;
	}

	// Try scrcpy-server first if enabled (higher quality, lower latency)
	if (scrcpyEnabled && scrcpyAvailable) {
		addLog("info", `[MIRROR] Trying scrcpy-server mode for ${serial}...`);
		const scrcpySuccess = await tryStartScrcpyServerStream(serial);
		if (scrcpySuccess) {
			return true;
		}
		addLog("warn", `[MIRROR] scrcpy-server mode failed, trying ADB mode...`);
	}

	// Fall back to ADB screenrecord
	addLog("info", `[MIRROR] Trying ADB screenrecord mode for ${serial}...`);
	try {
		videoStream = new VideoStream(serial);
		const frameRendered = await videoStream.start();

		if (frameRendered) {
			addLog("info", `[ADB] Video stream active for ${serial}`);
			return true;
		} else {
			addLog("warn", `[ADB] Video stream timeout, will use screenshots`);
			videoStream.stop();
			videoStream = null;
			return false;
		}
	} catch (error) {
		addLog("warn", `[ADB] Video stream unavailable: ${error}`);
		if (videoStream) {
			videoStream.stop();
			videoStream = null;
		}
		return false;
	}
}

/**
 * Try to start scrcpy-server streaming.
 * Returns true if first frame rendered, false otherwise.
 */
async function tryStartScrcpyServerStream(serial: string): Promise<boolean> {
	try {
		scrcpyServerStream = new ScrcpyVideoStream(serial);
		const frameRendered = await scrcpyServerStream.start();

		if (frameRendered) {
			addLog("info", `[SCRCPY] scrcpy-server stream active for ${serial}`);
			return true;
		} else {
			addLog("warn", `[SCRCPY] scrcpy-server timeout`);
			scrcpyServerStream.stop();
			scrcpyServerStream = null;
			return false;
		}
	} catch (error) {
		addLog("warn", `[SCRCPY] scrcpy-server unavailable: ${error}`);
		if (scrcpyServerStream) {
			scrcpyServerStream.stop();
			scrcpyServerStream = null;
		}
		return false;
	}
}

async function refreshMirrorScreen(serial: string): Promise<void> {
	const mirrorScreen = document.getElementById(
		"mirror-screen",
	) as HTMLImageElement;
	const secureNotice = document.getElementById("mirror-secure-notice");
	if (!mirrorScreen || !currentMirroringDevice) return;

	// If any video stream is active (ADB or scrcpy), skip screenshot polling
	if (videoStream || scrcpyServerStream) {
		return;
	}

	try {
		const base64 = await invoke<string>("capture_screenshot", { serial });
		if (base64) {
			// Check if screenshot is likely a secure/black screen
			// Sample less frequently to avoid performance impact
			const shouldCheckSecure = refreshFrameCount % 5 === 0;
			if (shouldCheckSecure) {
				const isBlack = await checkIfBlackScreen(base64);
				if (isBlack) {
					consecutiveBlackFrames++;
					// Only show notice after 3 consecutive black checks (sustained black)
					if (consecutiveBlackFrames >= 3 && secureNotice) {
						addLog("debug", "Screenshot consistently black - secure screen");
						secureNotice.style.display = "block";
						setMirrorModeBadge("secure");
					}
				} else {
					// Non-black frame - reset counter and hide notice
					consecutiveBlackFrames = 0;
					if (secureNotice) secureNotice.style.display = "none";
					setMirrorModeBadge("adb");
				}
			}
			mirrorScreen.src = `data:image/png;base64,${base64}`;
			mirrorErrorCount = 0; // Reset error count on success
		} else {
			// Screenshot returned empty - likely a secure screen (PIN/pattern)
			addLog("debug", "Screenshot returned empty - secure screen");
			consecutiveBlackFrames = 0; // Don't count empty as black
			if (secureNotice) secureNotice.style.display = "block";
		}
	} catch (error) {
		addLog("warn", `Screenshot failed: ${error}`);
		mirrorErrorCount++;
		if (mirrorErrorCount >= MAX_SCREENSHOT_ERRORS) {
			addLog(
				"error",
				"Device screenshot failed, stopping mirror. Check USB debugging authorization.",
			);
			stopMirror();
		}
	}
	refreshFrameCount++;
}

/**
 * Check if a base64 screenshot appears to be a black/blank screen
 * Used to detect secure screens that return valid but black images.
 * Uses variance-based detection to avoid false positives from dark themes.
 */
async function checkIfBlackScreen(base64: string): Promise<boolean> {
	return new Promise((resolve) => {
		const img = new Image();
		img.onload = () => {
			if (img.width === 0 || img.height === 0) {
				resolve(false);
				return;
			}
			const canvas = document.createElement("canvas");
			canvas.width = img.width;
			canvas.height = img.height;
			const ctx = canvas.getContext("2d");
			if (!ctx) {
				resolve(false);
				return;
			}
			ctx.drawImage(img, 0, 0);
			const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
			const data = imageData.data;

			// Use 5x5 grid of sample points to better detect blank screens
			// while ignoring dark themes and status bars
			const sampleXs = [0.1, 0.3, 0.5, 0.7, 0.9];
			const sampleYs = [0.2, 0.4, 0.5, 0.6, 0.8];
			const brightnesses: number[] = [];

			for (const fy of sampleYs) {
				for (const fx of sampleXs) {
					const x = Math.floor(fx * canvas.width);
					const y = Math.floor(fy * canvas.height);
					const idx = (y * canvas.width + x) * 4;
					const r = data[idx] ?? 0;
					const g = data[idx + 1] ?? 0;
					const b = data[idx + 2] ?? 0;
					// Calculate perceived brightness
					const brightness = r * 0.299 + g * 0.587 + b * 0.114;
					brightnesses.push(brightness);
				}
			}

			if (brightnesses.length === 0) {
				resolve(false);
				return;
			}

			const avgBrightness =
				brightnesses.reduce((a, b) => a + b, 0) / brightnesses.length;
			const maxBrightness = Math.max(...brightnesses);

			// True blank/black screen: ALL sample points are nearly black AND no variation
			// Dark themes: avg > 20 OR max > 30 (some bright pixels in icons/text)
			const isBlankScreen = avgBrightness < 3 && maxBrightness < 8;

			resolve(isBlankScreen);
		};
		img.onerror = () => resolve(false);
		img.src = `data:image/png;base64,${base64}`;
	});
}

/**
 * Internal stopMirror that resets state and UI but doesn't log
 * (used when switching between devices)
 */
async function stopMirrorInternal(): Promise<void> {
	if (!currentMirroringDevice) return;

	const deviceSerial = currentMirroringDevice;

	// Stop ADB video stream
	if (videoStream) {
		videoStream.stop();
		videoStream = null;
	}

	// Stop scrcpy-server video stream
	if (scrcpyServerStream) {
		scrcpyServerStream.stop();
		scrcpyServerStream = null;
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

	// Hide secure notice when stopping mirror
	const secureNotice = document.getElementById("mirror-secure-notice");
	if (secureNotice) secureNotice.style.display = "none";

	setMirrorModeBadge("disconnected");
	currentMirroringDevice = null;
}

async function stopMirror(): Promise<void> {
	if (!currentMirroringDevice) return;

	const deviceSerial = currentMirroringDevice;
	addLog("info", `Stopping mirror for ${deviceSerial}...`);

	// Use internal stop (no logging)
	await stopMirrorInternal();

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

	// Screen gesture handler for built-in mirror - use pointer events for tap and swipe
	// Pointer handlers must be bound to BOTH the screenshot <img> and the video
	// <canvas>: only one is visible at a time, and whichever is on top receives
	// the events. Without the canvas binding, taps/swipes are dead in scrcpy mode.
	const mirrorScreen = document.getElementById("mirror-screen");
	const mirrorScreenCanvas = document.getElementById("mirror-screen-canvas");
	for (const el of [mirrorScreen, mirrorScreenCanvas]) {
		if (!el) continue;
		el.addEventListener("pointerdown", handlePointerDown);
		el.addEventListener("pointermove", handlePointerMove);
		el.addEventListener("pointerup", handlePointerUp);
		el.addEventListener("pointercancel", handlePointerCancel);
		el.addEventListener("pointerleave", () => {
			// Don't cancel here - pointer capture keeps us receiving events
		});
	}

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

	// Device name edit
	const btnEditName = document.getElementById("btn-edit-device-name");
	const deviceNameSpan = document.getElementById("mirror-device-name");
	btnEditName?.addEventListener("click", () => {
		startDeviceNameEdit();
	});
	deviceNameSpan?.addEventListener("click", () => {
		startDeviceNameEdit();
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

	// Build and mount UI
	const app = document.getElementById("app")!;
	app.appendChild(build());
	setupEventListeners();

	// Check if user is logged in
	try {
		const user = await invoke<[string, string] | null>("get_user_info");
		if (user) {
			userInfo = { id: user[0], email: user[1] };
			updateUserBadgeFull();
			addLog(
				"info",
				`Logged in as ${userInfo.email} (${getCompanionVersionLabel()})`,
			);
			const loginSection = document.getElementById("login-section");
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
		display = computeDisplay(); // Recompute display state from new status
		addLog("debug", `Status update received: running=${state.agent_running}`);
		refreshUI();
	});

	// Listen for login success event from OAuth flow
	await listen<{ user_id: string; email: string }>("login-success", (event) => {
		userInfo = { id: event.payload.user_id, email: event.payload.email };
		updateUserBadgeFull();
		addLog(
			"info",
			`Signed in as ${userInfo.email} (${getCompanionVersionLabel()})`,
		);
		const loginSection = document.getElementById("login-section");
		const mainContent = document.getElementById("main-content");
		if (loginSection) loginSection.style.display = "none";
		if (mainContent) mainContent.style.display = "flex";
		// Refresh status to update device list
		refreshStatus();
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
