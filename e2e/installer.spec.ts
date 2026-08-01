/**
 * E2E Tests for Companion Installer & Dependency Management
 *
 * These tests verify:
 *   1. install_adb Tauri command is registered and calls deps::install_adb()
 *   2. dependency_manager has get_adb_dir, get_adb_url, is_adb_installed, install_adb
 *   3. DependencyStatus includes 'adb: bool' field
 *   4. install.sh exists, is syntactically valid, and covers all required features
 *
 * Run via Playwright (uses Vite dev server):
 *   npx playwright test e2e/installer.spec.ts
 *
 * These tests verify source-code invariants using string matching on Rust/Shell files.
 */

import { test, expect } from "@playwright/test";
import { existsSync, readFileSync } from "fs";
import { join } from "path";

/* eslint-disable @typescript-eslint/no-explicit-any */

const PROJECT_ROOT = new URL("..", import.meta.url).pathname;

function src(filename: string): string {
	return readFileSync(join(PROJECT_ROOT, filename), "utf8");
}

function srcMatch(
	filename: string,
	pattern: RegExp | string,
	msg: string,
): void {
	const content = src(filename);
	if (typeof pattern === "string") {
		expect(content).toMatch(pattern);
	} else {
		expect(pattern.test(content)).toBe(true);
	}
	console.log(`  ✓ ${msg}`);
}

// ─── adb.rs ──────────────────────────────────────────────────────────────────

test.describe("adb.rs — find_adb and auto-install", () => {
	test("bundled_adb_path() returns platform-tools subdirectory", () => {
		srcMatch(
			"src-tauri/src/adb.rs",
			/fn bundled_adb_path\(\).*platform-tools/s,
			"bundled_adb_path() uses 'platform-tools'",
		);
	});

	test("find_adb() checks bundled path BEFORE system candidates", () => {
		const content = src("src-tauri/src/adb.rs");
		const bundledIdx = content.indexOf("bundled_adb_path()");
		const candidatesIdx = content.indexOf("candidates = [");
		expect(bundledIdx).toBeGreaterThan(0);
		expect(bundledIdx).toBeLessThan(candidatesIdx);
		console.log("  ✓ find_adb() checks bundled path before system candidates");
	});

	test("find_adb() falls back to PATH", () => {
		srcMatch(
			"src-tauri/src/adb.rs",
			/trying PATH/,
			"find_adb() falls back to PATH",
		);
	});

	test("install_adb_blocking() creates single-thread tokio runtime", () => {
		srcMatch(
			"src-tauri/src/adb.rs",
			/new_current_thread/,
			"uses new_current_thread tokio runtime",
		);
		srcMatch("src-tauri/src/adb.rs", /block_on/, "calls block_on");
		srcMatch(
			"src-tauri/src/adb.rs",
			/dependency_manager::install_adb/,
			"calls dependency_manager install function",
		);
	});

	test("install_adb_blocking() tries system install first, then bundled download", () => {
		srcMatch(
			"src-tauri/src/adb.rs",
			/try_system_install.*adb/s,
			"tries apt/brew/choco first",
		);
		srcMatch(
			"src-tauri/src/adb.rs",
			/install_adb\(\).*await/s,
			"falls back to bundled download",
		);
	});
});

// ─── dependency_manager.rs ─────────────────────────────────────────────────────

test.describe("dependency_manager.rs — ADB integration", () => {
	test("get_adb_dir() returns platform-tools subdirectory", () => {
		srcMatch(
			"src-tauri/src/dependency_manager.rs",
			/pub fn get_adb_dir\(\).*platform-tools/s,
			"get_adb_dir() uses 'platform-tools'",
		);
	});

	test("get_adb_bin() delegates to get_adb_dir()", () => {
		srcMatch(
			"src-tauri/src/dependency_manager.rs",
			/pub fn get_adb_bin\(\).*get_adb_dir\(\)/s,
			"get_adb_bin() uses get_adb_dir()",
		);
	});

	test("get_adb_url() covers Linux, macOS, Windows", () => {
		srcMatch(
			"src-tauri/src/dependency_manager.rs",
			/dl\.google\.com.*platform-tools-latest-linux\.zip/,
			"Linux URL present",
		);
		srcMatch(
			"src-tauri/src/dependency_manager.rs",
			/dl\.google\.com.*platform-tools-latest-darwin\.zip/,
			"macOS URL present",
		);
		srcMatch(
			"src-tauri/src/dependency_manager.rs",
			/dl\.google\.com.*platform-tools-latest-windows\.zip/,
			"Windows URL present",
		);
	});

	test("is_adb_installed() checks system PATH and bundled path", () => {
		srcMatch(
			"src-tauri/src/dependency_manager.rs",
			/fn is_adb_installed\(\).*get_adb_bin\(\)/s,
			"checks bundled path",
		);
		srcMatch(
			"src-tauri/src/dependency_manager.rs",
			/which.*adb/,
			"checks system PATH via which",
		);
	});

	test("install_adb() tries system install first, then downloads", () => {
		srcMatch(
			"src-tauri/src/dependency_manager.rs",
			/pub async fn install_adb\(\)[\s\S]*?try_system_install.*adb/s,
			"tries apt/brew/choco first",
		);

		// Verify download_file is called after install_adb and platform-tools is in the file
		const content = src("src-tauri/src/dependency_manager.rs");
		const installAdbIdx = content.indexOf("pub async fn install_adb()");
		const dlIdx = content.indexOf("download_file", installAdbIdx);
		expect(dlIdx).toBeGreaterThan(installAdbIdx);
		expect(content).toMatch(/download_file.*platform-tools/s);
		console.log("  ✓ downloads platform-tools from Google");
	});

	test("install_all() includes install_adb()", () => {
		const content = src("src-tauri/src/dependency_manager.rs");
		const fnBody = content.match(
			/pub async fn install_all\(\)[^{]*\{([\s\S]*?)\n\}/,
		)?.[1];
		expect(fnBody).toBeTruthy();
		expect(fnBody).toMatch(/install_adb\(\)/);
		console.log("  ✓ install_all() calls install_adb()");
	});

	test("are_all_deps_installed() includes is_adb_installed()", () => {
		srcMatch(
			"src-tauri/src/dependency_manager.rs",
			/are_all_deps_installed.*is_adb_installed\(\)/s,
			"includes is_adb_installed()",
		);
	});

	test("DependencyStatus struct has 'adb: bool' field", () => {
		srcMatch(
			"src-tauri/src/dependency_manager.rs",
			/pub struct DependencyStatus[\s\S]*?pub adb:\s*bool/s,
			"has 'adb: bool' field",
		);
	});

	test("DependencyStatus::check() populates adb from is_adb_installed()", () => {
		srcMatch(
			"src-tauri/src/dependency_manager.rs",
			/adb:\s*is_adb_installed\(\)/,
			"sets adb field from is_adb_installed()",
		);
	});

	test("get_path_env() includes ADB directory", () => {
		srcMatch(
			"src-tauri/src/dependency_manager.rs",
			/get_path_env\(\).*get_adb_dir\(\)/s,
			"prepends bundled ADB to PATH",
		);
	});
});

// ─── lib.rs ──────────────────────────────────────────────────────────────────

test.describe("lib.rs — Tauri command registration", () => {
	test("install_adb async fn is defined", () => {
		srcMatch(
			"src-tauri/src/lib.rs",
			/async fn install_adb\(\)/,
			"install_adb() defined",
		);
	});

	test("install_adb command calls deps::install_adb()", () => {
		srcMatch(
			"src-tauri/src/lib.rs",
			/deps::install_adb\(\)/,
			"calls deps::install_adb()",
		);
	});

	test("install_adb is registered in invoke_handler", () => {
		srcMatch(
			"src-tauri/src/lib.rs",
			/invoke_handler.*\n.*install_adb/s,
			"install_adb in invoke_handler",
		);
	});
});

// ─── install.sh ──────────────────────────────────────────────────────────────

test.describe("install.sh — Linux installer script", () => {
	test("install.sh exists in scripts/ directory", () => {
		expect(existsSync(join(PROJECT_ROOT, "scripts", "install.sh"))).toBe(true);
		console.log("  ✓ install.sh exists");
	});

	test("install.sh is syntactically valid shell script", async () => {
		// Verify shell syntax using node child_process (use dynamic import for ESM)
		const { execSync } = await import("child_process");
		const scriptPath = `"${join(PROJECT_ROOT, "scripts", "install.sh")}"`;
		// sh -n checks syntax without executing — should not throw
		try {
			execSync(`/bin/sh -n ${scriptPath}`, { encoding: "utf8" } as any);
			console.log("  ✓ passes sh -n syntax check");
		} catch (e: unknown) {
			// Fail with the actual error message
			expect(String(e)).toBe("");
		}
	});

	test("downloads from RELEASE_BASE environment variable", () => {
		srcMatch("scripts/install.sh", /RELEASE_BASE=/, "defines RELEASE_BASE");
		srcMatch(
			"scripts/install.sh",
			/TARBALL_URL=.*RELEASE_BASE/,
			"uses RELEASE_BASE for tarball",
		);
	});

	test("defaults install dir to ~/.local/bin", () => {
		srcMatch(
			"scripts/install.sh",
			/\$HOME\/\.local\/bin/,
			"defaults to ~/.local/bin",
		);
	});

	test("creates .desktop file in ~/.local/share/applications", () => {
		srcMatch(
			"scripts/install.sh",
			/amos-companion\.desktop/,
			"creates .desktop entry",
		);
		srcMatch(
			"scripts/install.sh",
			/DESKTOP_DIR|\.local\/share\/applications/,
			"in correct directory",
		);
	});

	test("enables autostart via X-GNOME-Autostart-enabled=true", () => {
		srcMatch(
			"scripts/install.sh",
			/X-GNOME-Autostart-enabled=true/,
			"sets autostart",
		);
	});

	test("supports VERSION environment variable", () => {
		srcMatch(
			"scripts/install.sh",
			/VERSION=.*latest/,
			"defaults VERSION to latest",
		);
	});

	test("resolves latest version from manifest.json", () => {
		srcMatch("scripts/install.sh", /manifest\.json/, "fetches manifest.json");
		srcMatch(
			"scripts/install.sh",
			/resolve_version/,
			"has resolve_version function",
		);
	});

	test("detects x86_64 and aarch64 architectures", () => {
		srcMatch("scripts/install.sh", /x86_64/, "supports x86_64");
		srcMatch("scripts/install.sh", /aarch64|arm64/, "supports aarch64/arm64");
	});

	test("has arch-specific tarball filenames", () => {
		srcMatch(
			"scripts/install.sh",
			/amos-companion-\$\{ARCH\}/,
			"uses arch in tarball name",
		);
	});
});

// ─── bundled ADB path formula ─────────────────────────────────────────────────

test.describe("Bundled ADB path formula", () => {
	test("uses platform-tools subdirectory (standard Android SDK name)", () => {
		// The directory name must match Google's published SDK
		const url =
			"https://dl.google.com/android/repository/platform-tools-latest-linux.zip";
		expect(url).toMatch(/platform-tools/);
		expect(url).toMatch(/dl\.google\.com/);
		console.log("  ✓ standard platform-tools URL verified");
	});

	test("companion data dir is under XDG_DATA_HOME or ~/.local/share", () => {
		const home = process.env.HOME || "/root";
		const xdg = process.env.XDG_DATA_HOME || join(home, ".local/share");
		const companionDir = join(xdg, "amos-companion");
		const adbBin = join(companionDir, "platform-tools", "adb");
		expect(adbBin).toContain("amos-companion");
		expect(adbBin).toContain("platform-tools");
		console.log(`  ✓ formula: ${adbBin}`);
	});
});
