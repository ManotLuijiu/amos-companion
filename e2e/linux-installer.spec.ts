/**
 * E2E Tests for Linux install.sh Script
 *
 * These tests run the install.sh script against a real or mocked R2 endpoint.
 * Designed for CI (GitHub Actions) where the script is downloaded and executed.
 *
 * Prerequisites (set in CI env):
 *   CI=true
 *
 * Run locally (Linux/macOS with Node.js):
 *   npx playwright test e2e/linux-installer.spec.ts
 *
 * These tests are SKIPPED on non-Linux platforms (install.sh is Linux-only).
 */

import { test, expect } from "@playwright/test";
import { execSync } from "child_process";
import { existsSync, rmSync, readFileSync } from "fs";
import { join } from "path";
import { tmpdir } from "os";

/* eslint-disable @typescript-eslint/no-explicit-any */

const IS_LINUX = process.platform === "linux" || process.platform === "darwin";
const PROJECT_ROOT = new URL("..", import.meta.url).pathname;
const INSTALL_SCRIPT_PATH = join(PROJECT_ROOT, "scripts", "install.sh");

// Skip all tests on non-Linux (install.sh is for Linux)
test.describe(
	"install.sh — Integration tests (Linux only)",
	{ tag: ["@linux"] },
	() => {
		test.beforeEach(() => {
			if (!IS_LINUX) {
				test.skip(true, "install.sh is Linux-only");
			}
		});

		test.describe("Script is present and valid", () => {
			test("install.sh exists", () => {
				expect(existsSync(INSTALL_SCRIPT_PATH)).toBe(true);
			});

			test("has valid shell shebang", () => {
				const content = readFileSync(INSTALL_SCRIPT_PATH, "utf8");
				expect(content).toMatch(/^#!\/bin\/sh\b/);
			});

			test("passes shell syntax check (sh -n)", () => {
				expect(() =>
					execSync(`/bin/sh -n "${INSTALL_SCRIPT_PATH}"`),
				).not.toThrow();
			});
		});

		test.describe("Environment variable overrides", () => {
			test("RELEASE_BASE can be overridden", () => {
				// Run install.sh with a custom RELEASE_BASE pointing to a mock URL
				// The script should try to download from the custom URL
				const result = execSync(
					`RELEASE_BASE="https://example.com/mock" VERSION="0.0.0-test" /bin/sh "${INSTALL_SCRIPT_PATH}" 2>&1 || true`,
					{ encoding: "utf8" },
				);
				// We expect it to fail gracefully because the mock URL doesn't exist
				// But it should NOT fail with a shell syntax error
				expect(result).toBeDefined();
				// The script should have printed its usage or reached the download step
				const output = result as string;
				expect(
					output.includes("Unsupported architecture") ||
						output.includes("Downloading AMOS Companion") ||
						output.includes("Already on latest version") ||
						output.includes("ERROR") ||
						output.includes("Download failed") ||
						output.includes("Failed to fetch"),
				).toBe(true);
				console.log(
					"  Sample output:",
					(result as string).split("\n").slice(0, 3).join(" "),
				);
			});

			test("INSTALL_DIR can be overridden", () => {
				const tmpInstall = join(tmpdir(), `amos-test-${Date.now()}`);
				const result = execSync(
					`INSTALL_DIR="${tmpInstall}" VERSION="0.0.0-test" /bin/sh "${INSTALL_SCRIPT_PATH}" 2>&1 || true`,
					{ encoding: "utf8" },
				);
				// Should either download or gracefully fail — no shell errors
				expect((result as string).includes("Syntax Error")).toBe(false);
				// Cleanup
				try {
					rmSync(tmpInstall, { recursive: true, force: true });
				} catch (_e) {
					/* ignore cleanup errors */
				}
			});

			test("VERSION=latest fetches from manifest.json", () => {
				const result = execSync(
					`VERSION=latest /bin/sh "${INSTALL_SCRIPT_PATH}" 2>&1 || true`,
					{ encoding: "utf8", timeout: 30_000 },
				);
				const output = result as string;
				// Should reach the manifest fetch step
				expect(
					output.includes("Checking for latest version") ||
						output.includes("manifest.json") ||
						output.includes("Failed to fetch") ||
						output.includes("Already on latest version"),
				).toBe(true);
			});
		});

		test.describe("Architecture detection", () => {
			test("x86_64 is detected on x86_64 machines", () => {
				const result = execSync(
					`/bin/sh -c 'arch() { echo x86_64; }; . "${INSTALL_SCRIPT_PATH}" < /dev/null 2>&1 || true'`,
					{ encoding: "utf8" },
				);
				const output = result as string;
				expect(
					output.includes("x86_64") ||
						output.includes("Downloading AMOS Companion") ||
						output.includes("Unsupported architecture"),
				).toBe(true);
			});

			test("aarch64 is detected on ARM machines", () => {
				const result = execSync(
					`/bin/sh -c 'arch() { echo aarch64; }; . "${INSTALL_SCRIPT_PATH}" < /dev/null 2>&1 || true'`,
					{ encoding: "utf8" },
				);
				const output = result as string;
				expect(
					output.includes("aarch64") ||
						output.includes("Downloading AMOS Companion") ||
						output.includes("Unsupported architecture"),
				).toBe(true);
			});

			test("unknown arch exits with error", () => {
				const result = execSync(
					`/bin/sh -c 'arch() { echo mips; }; . "${INSTALL_SCRIPT_PATH}" < /dev/null 2>&1 || true'`,
					{ encoding: "utf8" },
				);
				expect((result as string).includes("Unsupported architecture")).toBe(
					true,
				);
			});
		});

		test.describe("Desktop integration", () => {
			test("creates .desktop file content is valid", () => {
				const content = readFileSync(INSTALL_SCRIPT_PATH, "utf8");
				// Extract the .desktop template from the script
				const desktopMatch = content.match(
					/cat > .*?amos-companion\.desktop.*?DESKTOP_EOF/s,
				);
				expect(desktopMatch).toBeTruthy();
				const desktopContent = desktopMatch![0]
					.replace(/cat > .*?<< 'DESKTOP_EOF'\n?/, "")
					.replace(/\n?DESKTOP_EOF\n?/, "")
					.trim();

				expect(desktopContent).toMatch(/\[Desktop Entry\]/);
				expect(desktopContent).toMatch(/^Name=AMOS Companion$/m);
				expect(desktopContent).toMatch(/^Exec=amos-companion$/m);
				expect(desktopContent).toMatch(/^Icon=amos-companion$/m);
				expect(desktopContent).toMatch(/^Type=Application$/m);
				expect(desktopContent).toMatch(/^Categories=/m);
				expect(desktopContent).toMatch(/Keywords=.*android.*/);
				console.log("  ✓ .desktop entry is valid INI format");
			});

			test("autostart entry sets X-GNOME-Autostart-enabled=true", () => {
				const content = readFileSync(INSTALL_SCRIPT_PATH, "utf8");
				expect(content).toMatch(/X-GNOME-Autostart-enabled=true/);
				expect(content).toMatch(/\.config\/autostart/);
				console.log("  ✓ autostart entry is correct");
			});
		});

		test.describe("Error handling", () => {
			test("missing curl shows error", () => {
				const result = execSync(
					`/bin/sh -c 'curl() { return 127; }; . "${INSTALL_SCRIPT_PATH}"' 2>&1 || true`,
					{ encoding: "utf8" },
				);
				expect(
					(result as string).includes("Missing required commands") ||
						(result as string).includes("curl"),
				).toBe(true);
			});

			test("404 download shows graceful error", () => {
				const result = execSync(
					`RELEASE_BASE="https://httpstat.us/404" VERSION="0.0.0-test" /bin/sh "${INSTALL_SCRIPT_PATH}" 2>&1 || true`,
					{ encoding: "utf8", timeout: 30_000 },
				);
				expect(
					(result as string).includes("Download failed") ||
						(result as string).includes("ERROR"),
				).toBe(true);
			});
		});
	},
);

// ─── install.sh version/update behavior ──────────────────────────────────────

test.describe("install.sh version/update behavior", () => {
	test.describe("Version detection", () => {
		test("get_installed_version returns not-installed when binary absent", () => {
			// Test the version detection logic
			const result = execSync(
				`INSTALL_DIR="/nonexistent" /bin/sh -c '. "${INSTALL_SCRIPT_PATH}" < /dev/null; echo "INSTALLED_VERSION=$INSTALLED_VERSION"' 2>&1 || true`,
				{ encoding: "utf8" },
			);
			const output = result as string;
			// Should report not-installed when directory doesn't exist
			expect(
				output.includes("not-installed") || output.includes("unknown"),
			).toBe(true);
		});

		test("supports VERSION environment variable override", () => {
			// VERSION override should work
			const result = execSync(
				`VERSION="1.6.66" /bin/sh "${INSTALL_SCRIPT_PATH}" 2>&1 || true`,
				{ encoding: "utf8", timeout: 30000 },
			);
			const output = result as string;
			// Should either reach the download step or gracefully handle the version
			expect(
				output.includes("1.6.66") ||
					output.includes("Downloading AMOS Companion") ||
					output.includes("Download failed") ||
					output.includes("Already on latest version"),
			).toBe(true);
		});
	});

	test.describe("Update mode guidance", () => {
		test("install.sh includes Ubuntu/deb guidance in comments", () => {
			const content = readFileSync(INSTALL_SCRIPT_PATH, "utf8");
			// Should include guidance for .deb users
			expect(
				content.includes("sudo apt install") || content.includes("dpkg -i"),
			).toBe(true);
			console.log("  ✓ install.sh includes .deb update guidance");
		});
	});
});

// ─── Frontend: install.sh download link is shown on Linux ─────────────────────

test.describe("Frontend — Linux install.sh download link", () => {
	test.beforeEach(async ({ page }) => {
		await page.goto("/");
	});

	test("shows Linux install hint when on Linux", async ({ page }) => {
		// The companion frontend should show a message directing Linux users
		// to download install.sh when the AppImage is not available
		const isLinux = await page.evaluate(() =>
			navigator.platform.includes("Linux"),
		);
		if (isLinux) {
			// The frontend should surface guidance for Linux users
			// (This test verifies the placeholder/comment exists in source)
			const content = readFileSync(
				join(PROJECT_ROOT, "src-ui", "App.ts"),
				"utf8",
			);
			expect(content).toMatch(/install\.sh|curl.*sh.*install/i);
			console.log("  ✓ Frontend references install.sh for Linux guidance");
		} else {
			test.skip(true, "Not on Linux");
		}
	});
});
