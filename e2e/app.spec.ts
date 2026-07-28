import { readFileSync } from "node:fs";
import { test, expect } from "@playwright/test";

const { version: packageVersion } = JSON.parse(
	readFileSync(new URL("../package.json", import.meta.url), "utf8"),
) as { version: string };

/**
 * E2E Tests for AMOS Companion UI
 *
 * These tests verify the core UI elements are present.
 * Note: Full app tests (device control, mirroring) require Tauri app running.
 */

/**
 * Tests for editable device friendly name feature
 */
test.describe("Device Friendly Name Editing", () => {
	test.beforeEach(async ({ page }) => {
		await page.goto("/");
	});

	test("should have edit button in mirror header", async ({ page }) => {
		const editBtn = page.locator("#btn-edit-device-name");
		await expect(editBtn).toBeAttached();
	});

	test("should have device name span in mirror header", async ({ page }) => {
		const deviceName = page.locator("#mirror-device-name");
		await expect(deviceName).toBeAttached();
		await expect(deviceName).toContainText("Screen Mirror");
	});

	test("edit button should be hidden initially", async ({ page }) => {
		const editBtn = page.locator("#btn-edit-device-name");
		await expect(editBtn).toHaveCSS("opacity", "0");
	});

	test("edit button should be visible on header hover", async ({ page }) => {
		const editBtn = page.locator("#btn-edit-device-name");
		const header = page.locator(".mirror-header");
		await header.hover();
		await expect(editBtn).toHaveCSS("opacity", "1");
	});

	test("should have CSS class for device name input styling", async ({
		page,
	}) => {
		// Verify the CSS class exists in stylesheet
		const hasInputStyle = await page.evaluate(() => {
			return Array.from(document.styleSheets)
				.filter((s) => s.cssRules.length > 0)
				.flatMap((s) => Array.from(s.cssRules))
				.some((r) => r.cssText.includes("device-name-input"));
		});
		expect(hasInputStyle).toBeTruthy();
	});
});

test.describe("AMOS Companion UI", () => {
	test.beforeEach(async ({ page }) => {
		await page.goto("/");
	});

	test("should load the app with correct title", async ({ page }) => {
		await expect(page).toHaveTitle(/AMOS Companion/);
	});

	test("should display header with logo and title", async ({ page }) => {
		await expect(page.locator(".header-title")).toContainText("AMOS Companion");

		const logo = page.locator(".header-logo");
		await expect(logo).toBeVisible();
		expect(
			await logo.evaluate((img) => (img as HTMLImageElement).naturalWidth > 0),
		).toBeTruthy();
	});

	test("should display the current package version in the header", async ({
		page,
	}) => {
		await expect(page.locator(".header-version")).toHaveText(
			`v${packageVersion}`,
		);
	});

	test("should show login section or main content", async ({ page }) => {
		const loginSection = page.locator("#login-section");
		const mainContent = page.locator("#main-content");
		const loginVisible = await loginSection.isVisible();
		const mainVisible = await mainContent.isVisible();
		expect(loginVisible || mainVisible).toBeTruthy();
	});

	test("should display status badge", async ({ page }) => {
		await expect(page.locator("#status-badge")).toBeVisible();
	});

	test("should show activity logs panel", async ({ page }) => {
		await expect(page.locator("#log-container")).toBeVisible();
	});

	test("should have log filter with options", async ({ page }) => {
		const filter = page.locator("#log-filter");
		await expect(filter).toBeVisible();
		await expect(filter.locator("option")).toHaveCount(4);
	});

	test("should have settings card with API URL input", async ({ page }) => {
		await expect(page.locator("#api-url")).toBeAttached();
	});

	test("should have Open Web UI button", async ({ page }) => {
		await expect(page.locator("#btn-open-web")).toBeVisible();
	});

	test("should have three-panel layout", async ({ page }) => {
		await expect(page.locator(".panel-left")).toBeAttached();
		await expect(page.locator(".panel-mirror")).toBeAttached();
		await expect(page.locator(".panel-right")).toBeAttached();
	});

	test("should display footer", async ({ page }) => {
		await expect(page.locator(".app-footer")).toContainText(
			"AMOS Device Management",
		);
	});

	test("should show placeholder in mirror panel when no device selected", async ({
		page,
	}) => {
		await expect(page.locator("#mirror-placeholder")).toContainText(
			"Select a device to start mirroring",
		);
	});

	test("should have scrcpy toggle checkbox", async ({ page }) => {
		await expect(page.locator("#toggle-scrcpy")).toBeAttached();
	});

	test("should have device search input", async ({ page }) => {
		await expect(page.locator("#device-search")).toBeAttached();
	});
});

// Note: Agent control tests (start/stop button, device list) require login
// These can be tested when we implement auth mocking in E2E tests

// Note: Secure-screen notice and device online indicator are UI-only features
// that require actual device mirroring to test fully in E2E environment
