#!/usr/bin/env node
/**
 * AMOS Companion CLI
 *
 * One-stop command to connect Android devices to AMOS SaaS.
 *
 * Usage:
 *   npx amos-connect                    # Interactive setup
 *   amos-connect --token <TOKEN>        # Non-interactive with API token
 *   amos-connect --configure            # Configure settings
 *   amos-connect --status               # Show connection status
 *   amos-connect --uninstall            # Remove from system
 */

import { spawn } from "child_process";
import { readFileSync, writeFileSync, mkdirSync } from "fs";
import { homedir } from "os";
import { join } from "path";
import { createInterface } from "readline";

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

interface Config {
	apiUrl: string;
	apiToken: string;
	agentId: string;
	intervalSeconds: number;
	dryRun: boolean;
	verifySsl: boolean;
}

const CONFIG_PATH = join(homedir(), ".amos", "config.json");

function loadConfig(): Config | null {
	try {
		return JSON.parse(readFileSync(CONFIG_PATH, "utf-8"));
	} catch {
		return null;
	}
}

function saveConfig(config: Config): void {
	mkdirSync(join(homedir(), ".amos"), { recursive: true });
	writeFileSync(CONFIG_PATH, JSON.stringify(config, null, 2));
	console.log(`✅ Config saved to ${CONFIG_PATH}`);
}

// ---------------------------------------------------------------------------
// CLI parser
// ---------------------------------------------------------------------------

function parseArgs(argv: string[]): {
	action: "start" | "configure" | "status" | "uninstall";
	options: Partial<Config>;
} {
	const args = argv.slice(2);
	let action: "start" | "configure" | "status" | "uninstall" = "start";
	const options: Partial<Config> = {};

	for (let i = 0; i < args.length; i++) {
		const arg = args[i];
		if (arg === "--configure" || arg === "-c") action = "configure";
		else if (arg === "--status" || arg === "-s") action = "status";
		else if (arg === "--uninstall" || arg === "-u") action = "uninstall";
		else if (arg === "--token" || arg === "-t") {
			options.apiToken = args[++i];
		} else if (arg === "--api-url") {
			options.apiUrl = args[++i];
		} else if (arg === "--agent-id") {
			options.agentId = args[++i];
		} else if (arg === "--dry-run") {
			options.dryRun = true;
		} else if (arg === "--interval") {
			options.intervalSeconds = parseFloat(args[++i]);
		} else if (arg === "--no-verify-ssl") {
			options.verifySsl = false;
		} else if (arg === "--help" || arg === "-h") {
			printHelp();
			process.exit(0);
		}
	}

	return { action, options };
}

// ---------------------------------------------------------------------------
// ADB discovery
// ---------------------------------------------------------------------------

async function checkAdb(): Promise<boolean> {
	return new Promise((resolve) => {
		const proc = spawn("adb", ["version"], { shell: true });
		proc.on("close", (code) => resolve(code === 0));
		proc.on("error", () => resolve(false));
	});
}

async function listDevices(): Promise<string[]> {
	return new Promise((resolve) => {
		const proc = spawn("adb", ["devices"], { shell: true, encoding: "utf-8" });
		let output = "";
		proc.stdout.on("data", (data) => (output += data));
		proc.on("close", () => {
			const lines = output
				.split("\n")
				.filter((l) => l.trim() && !l.startsWith("List"));
			const devices = lines.map((l) => l.split("\t")[0].trim()).filter(Boolean);
			resolve(devices);
		});
		proc.on("error", () => resolve([]));
	});
}

// ---------------------------------------------------------------------------
// API client
// ---------------------------------------------------------------------------

async function apiGet(url: string, token: string): Promise<unknown> {
	const { default: axios } = await import("axios");
	const resp = await axios.get(url, {
		headers: { Authorization: `Bearer ${token}` },
		timeout: 5000,
		validateStatus: () => true,
	});
	return resp.data;
}

async function apiPost(
	url: string,
	token: string,
	data: unknown,
): Promise<unknown> {
	const { default: axios } = await import("axios");
	const resp = await axios.post(url, data, {
		headers: { Authorization: `Bearer ${token}` },
		timeout: 5000,
		validateStatus: () => true,
	});
	return resp.data;
}

async function checkApiHealth(apiUrl: string): Promise<boolean> {
	try {
		const { default: axios } = await import("axios");
		await axios.get(`${apiUrl}/health`, { timeout: 5000 });
		return true;
	} catch {
		return false;
	}
}

// ---------------------------------------------------------------------------
// Token management
// ---------------------------------------------------------------------------

async function exchangeApiKeyForToken(
	apiUrl: string,
	apiKey: string,
	apiSecret: string,
): Promise<{ token: string; agentId: string }> {
	const data = (await apiPost(`${apiUrl}/auth/device-token`, "", {
		api_key: apiKey,
		api_secret: apiSecret,
	})) as { token?: string; agent_id?: string };
	if (!data.token || !data.agent_id) {
		throw new Error("Failed to exchange API key for token");
	}
	return { token: data.token, agentId: data.agent_id };
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

function runDeviceAgent(config: Config): void {
	const args = [
		"run",
		"python",
		"-m",
		"amos_device_agent",
		"--interval",
		String(config.intervalSeconds),
	];
	if (!config.verifySsl) args.push("--no-verify-ssl");

	console.log("🚀 Starting AMOS device-agent...");
	console.log(`   API URL: ${config.apiUrl}`);
	console.log(`   Agent ID: ${config.agentId}`);
	console.log(`   Interval: ${config.intervalSeconds}s`);
	console.log("");

	const env = {
		...process.env,
		AMOS_API_URL: config.apiUrl,
		AMOS_AGENT_ID: config.agentId,
		AMOS_API_TOKEN: config.apiToken,
		AMOS_SKIP_SSL_VERIFY: config.verifySsl ? "0" : "1",
	};

	// Find the Python venv in the auto-affiliate-agents repo
	const repoPaths = [
		join(homedir(), "auto-affiliate-agents/backend"),
		join(__dirname, "../../backend"),
	];
	let venvPython = "python3";
	for (const repoPath of repoPaths) {
		try {
			const testPath = join(repoPath, ".venv/bin/python");
			require("fs").accessSync(testPath);
			venvPython = testPath;
			break;
		} catch {
			// try next path
		}
	}

	const proc = spawn(venvPython, args.slice(1), {
		cwd: join(homedir(), "auto-affiliate-agents/backend/services/device-agent"),
		env,
		shell: true,
	});

	proc.stdout.on("data", (data) => process.stdout.write(data));
	proc.stderr.on("data", (data) => process.stderr.write(data));

	// Forward Ctrl+C / SIGTERM to the child process so it shuts down cleanly
	process.on("SIGINT", () => {
		console.error("\n\u23F9 Shutting down...");
		proc.kill("SIGINT");
	});
	process.on("SIGTERM", () => {
		console.error("\u23F9 Received SIGTERM, shutting down...");
		proc.kill("SIGTERM");
	});

	proc.on("close", (code) => {
		// Only exit if the child died unexpectedly (not from our own SIGINT/SIGTERM)
		console.error(`Device-agent exited with code ${code}`);
		process.exit(code ?? 1);
	});
}

// ---------------------------------------------------------------------------
// Interactive setup
// ---------------------------------------------------------------------------

async function prompt(question: string): Promise<string> {
	const rl = createInterface({ input: process.stdin, output: process.stdout });
	return new Promise((resolve) => {
		rl.question(question, (answer) => {
			rl.close();
			resolve(answer.trim());
		});
	});
}

async function interactiveSetup(existing?: Config): Promise<Config> {
	console.log("\n📱 AMOS Companion — Setup\n");

	const apiUrl =
		existing?.apiUrl ||
		(await prompt("AMOS API URL [https://amos-api.moo-vpn.online]: ")) ||
		"https://amos-api.moo-vpn.online";

	const hasApiKey = await prompt(
		"Do you have an API Key from the AMOS dashboard? (y/n): ",
	);
	let apiToken = existing?.apiToken || "";
	let agentId = existing?.agentId || "agent-local";

	if (hasApiKey.toLowerCase() === "y" || hasApiKey.toLowerCase() === "yes") {
		const apiKey = await prompt("Enter your API Key (ak_...): ");
		const apiSecret = await prompt("Enter your API Secret (sk_...): ");
		try {
			const result = await exchangeApiKeyForToken(apiUrl, apiKey, apiSecret);
			apiToken = result.token;
			agentId = result.agentId;
			console.log(`✅ Token obtained for agent: ${agentId}`);
		} catch (err) {
			console.error("❌ Failed to get token:", err);
			process.exit(1);
		}
	} else {
		apiToken = await prompt("Enter Bearer Token: ");
		agentId = (await prompt(`Agent ID [${agentId}]: `)) || agentId;
	}

	const verifySsl = await prompt("Verify SSL certificates? (y/n) [y]: ");
	const interval = await prompt("Heartbeat interval in seconds [30]: ");

	return {
		apiUrl,
		apiToken,
		agentId,
		intervalSeconds: parseFloat(interval) || 30,
		dryRun: false,
		verifySsl: verifySsl.toLowerCase() !== "n",
	};
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

async function actionConfigure(): Promise<void> {
	const existing = loadConfig();
	const config = await interactiveSetup(existing ?? undefined);
	saveConfig(config);
}

async function actionStart(options: Partial<Config>): Promise<void> {
	const existing = loadConfig();
	const config: Config = {
		apiUrl:
			existing?.apiUrl || options.apiUrl || "https://amos-api.moo-vpn.online",
		apiToken: existing?.apiToken || options.apiToken || "",
		agentId: existing?.agentId || options.agentId || "agent-local",
		intervalSeconds: existing?.intervalSeconds || options.intervalSeconds || 30,
		dryRun: existing?.dryRun || options.dryRun || false,
		verifySsl: existing?.verifySsl ?? options.verifySsl ?? true,
	};

	if (!config.apiToken) {
		console.error("❌ No API token configured. Run: amos-connect --configure");
		process.exit(1);
	}

	// Check API connectivity
	const healthy = await checkApiHealth(config.apiUrl);
	if (!healthy) {
		console.error(`❌ Cannot reach AMOS API at ${config.apiUrl}`);
		console.error("   Check your internet connection and API URL.");
		process.exit(1);
	}
	console.log(`✅ API is reachable: ${config.apiUrl}`);

	// Check ADB
	const adbAvailable = await checkAdb();
	if (!adbAvailable) {
		console.warn("⚠️  ADB not found. Install Android SDK Platform Tools.");
		console.warn(
			"   https://developer.android.com/studio/releases/platform-tools",
		);
	} else {
		console.log("✅ ADB is installed");
		const devices = await listDevices();
		if (devices.length > 0) {
			console.log(`📱 Connected devices: ${devices.join(", ")}`);
		} else {
			console.warn(
				"⚠️  No Android devices detected. Connect a tablet with USB debugging enabled.",
			);
		}
	}

	runDeviceAgent(config);
}

async function actionStatus(): Promise<void> {
	const config = loadConfig();
	if (!config) {
		console.log("❌ Not configured. Run: amos-connect --configure");
		return;
	}

	console.log("\n📊 AMOS Companion Status\n");
	console.log(`   API URL:    ${config.apiUrl}`);
	console.log(`   Agent ID:   ${config.agentId}`);
	console.log(`   Interval:    ${config.intervalSeconds}s`);
	console.log(`   SSL Verify: ${config.verifySsl ? "✅ Yes" : "❌ No"}`);
	console.log(`   Token:      ${config.apiToken ? "✅ Set" : "❌ Not set"}`);

	const healthy = await checkApiHealth(config.apiUrl);
	console.log(`   API:        ${healthy ? "✅ Reachable" : "❌ Unreachable"}`);

	const adbAvailable = await checkAdb();
	console.log(
		`   ADB:        ${adbAvailable ? "✅ Installed" : "❌ Not found"}`,
	);

	if (adbAvailable) {
		const devices = await listDevices();
		console.log(
			`   Devices:    ${devices.length > 0 ? devices.join(", ") : "None"}`,
		);
	}

	console.log("");
}

async function actionUninstall(): Promise<void> {
	const { rmSync } = await import("fs");
	try {
		rmSync(CONFIG_PATH);
		console.log("✅ Config removed from ~/.amos/config.json");
	} catch {
		console.log("No config file to remove.");
	}
	console.log(
		"\n⚠️  To fully uninstall, also remove the auto-affiliate-agents repository:",
	);
	console.log(`   rm -rf ${join(homedir(), "auto-affiliate-agents")}`);
}

// ---------------------------------------------------------------------------
// Help
// ---------------------------------------------------------------------------

function printHelp(): void {
	console.log(`
📱 AMOS Companion — Connect Android devices to AMOS SaaS

Usage:
  amos-connect [options]

Options:
  --configure          Interactive setup (or -c)
  --status            Show connection status (or -s)
  --uninstall         Remove config and stop agent (or -u)
  --token <TOKEN>     API token (non-interactive)
  --api-url <URL>     AMOS API URL
  --agent-id <ID>     Device agent ID
  --interval <SECS>   Heartbeat interval (default: 30)
  --dry-run           Run without real ADB
  --no-verify-ssl    Skip SSL certificate verification
  --help, -h          Show this help message

Examples:
  amos-connect --configure           # First-time setup
  amos-connect --token xyz --status  # Check status with token
  amos-connect --uninstall          # Remove from system

For more info: https://docs.amos.moo-vpn.online/companion
`);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main(): Promise<void> {
	const { action, options } = parseArgs(process.argv);

	switch (action) {
		case "configure":
			await actionConfigure();
			break;
		case "status":
			await actionStatus();
			break;
		case "uninstall":
			await actionUninstall();
			break;
		case "start":
		default:
			await actionStart(options);
			break;
	}
}

main().catch((err) => {
	console.error("❌ Error:", err);
	process.exit(1);
});
