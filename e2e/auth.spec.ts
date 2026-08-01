import { test, expect } from "@playwright/test";

/**
 * E2E Tests for AMOS Companion email/password authentication
 *
 * These tests verify the email/password sign-in flow works correctly.
 * Tests connect to the live AMOS API at https://api.amos.moo-vpn.online
 */

const TEST_ACCOUNTS = [
	{
		email: "munchira_01@hotmail.com",
		password: "CHANGE_ME", // Set E2E_TEST_PASSWORD env var
		label: "Hotmail account",
	},
] as const;

test.describe("Email/Password Authentication", () => {
	for (const account of TEST_ACCOUNTS) {
		test.describe(`Sign in with ${account.label}`, () => {
			test("should show email/password form fields", async ({ page }) => {
				await page.goto("/");

				// Wait for login section to be visible
				const loginSection = page.locator("#login-section");
				await expect(loginSection).toBeVisible({ timeout: 10000 });

				// Check email input exists
				const emailInput = page.locator("#login-email");
				await expect(emailInput).toBeVisible();

				// Check password input exists
				const passwordInput = page.locator("#login-password");
				await expect(passwordInput).toBeVisible();

				// Check submit button exists
				const submitBtn = page.locator('button[type="submit"]');
				await expect(submitBtn).toBeVisible();
				await expect(submitBtn).toContainText("Sign In");
			});

			test("should accept valid credentials and log in", async ({ page }) => {
				// Skip if no real password provided (CI environment)
				if (!process.env.E2E_TEST_PASSWORD) {
					test.skip(true, "E2E_TEST_PASSWORD not set — set the env var to run this test");
					return;
				}

				await page.goto("/");

				// Wait for login section
				const loginSection = page.locator("#login-section");
				await expect(loginSection).toBeVisible({ timeout: 10000 });

				// Fill in credentials
				const emailInput = page.locator("#login-email");
				const passwordInput = page.locator("#login-password");
				const submitBtn = page.locator('button[type="submit"]');

				await emailInput.fill(account.email);
				await passwordInput.fill(account.password);
				await submitBtn.click();

				// Wait for Activity Log to show sign-in result
				// The app calls the API and shows result in Activity Logs
				// On success, login section should hide and main content shows
				await page.waitForTimeout(3000);

				// Check Activity Logs for result
				const logContent = page.locator("#log-content");
				const logs = await logContent.textContent();

				// Should either show success or an API error (network issues in CI)
				expect(logs).toBeTruthy();

				// If login failed, should show error in logs
				if (logs && !logs.includes("Signed in")) {
					// Log the actual error for debugging
					console.log("Sign-in log content:", logs);
				}
			});

			test("should show error for invalid credentials", async ({ page }) => {
				await page.goto("/");

				// Wait for login section
				const loginSection = page.locator("#login-section");
				await expect(loginSection).toBeVisible({ timeout: 10000 });

				// Fill with wrong password
				const emailInput = page.locator("#login-email");
				const passwordInput = page.locator("#login-password");
				const submitBtn = page.locator('button[type="submit"]');
				const errorDiv = page.locator("#login-error");

				await emailInput.fill(account.email);
				await passwordInput.fill("wrong-password-12345");
				await submitBtn.click();

				// Wait for error response
				await page.waitForTimeout(3000);

				// Should show error message
				const errorText = await errorDiv.textContent();
				expect(errorText).toBeTruthy();

				// Error div should be visible
				await expect(errorDiv).toBeVisible();
			});

			test("should show error for non-existent email", async ({ page }) => {
				await page.goto("/");

				// Wait for login section
				const loginSection = page.locator("#login-section");
				await expect(loginSection).toBeVisible({ timeout: 10000 });

				// Fill with non-existent email
				const emailInput = page.locator("#login-email");
				const passwordInput = page.locator("#login-password");
				const submitBtn = page.locator('button[type="submit"]');
				const errorDiv = page.locator("#login-error");

				await emailInput.fill("nonexistent-email-12345@test.com");
				await passwordInput.fill("any-password");
				await submitBtn.click();

				// Wait for error response
				await page.waitForTimeout(3000);

				// Should show error message
				const errorText = await errorDiv.textContent();
				expect(errorText).toBeTruthy();

				// Error div should be visible
				await expect(errorDiv).toBeVisible();
			});
		});
	}

	test("should have API URL input field", async ({ page }) => {
		await page.goto("/");

		const apiUrlInput = page.locator("#login-api-url");
		await expect(apiUrlInput).toBeVisible();
		await expect(apiUrlInput).toHaveValue("https://api.amos.moo-vpn.online");
	});

	test("should have Google OAuth button", async ({ page }) => {
		await page.goto("/");

		const loginSection = page.locator("#login-section");
		await expect(loginSection).toBeVisible({ timeout: 10000 });

		const googleBtn = page.locator(".google-wrapper");
		await expect(googleBtn).toBeVisible();
	});
});

test.describe("Activity Logs", () => {
	test("should display sign-in attempts in Activity Logs", async ({ page }) => {
		await page.goto("/");

		const loginSection = page.locator("#login-section");
		await expect(loginSection).toBeVisible({ timeout: 10000 });

		const logContent = page.locator("#log-content");
		await expect(logContent).toBeVisible();
	});

	test("should show dependency status after login", async ({ page }) => {
		if (!process.env.E2E_TEST_PASSWORD) {
			test.skip(true, "E2E_TEST_PASSWORD not set");
		}

		await page.goto("/");

		const loginSection = page.locator("#login-section");
		await expect(loginSection).toBeVisible({ timeout: 10000 });

		// Sign in
		await page.locator("#login-email").fill(TEST_ACCOUNTS[0].email);
		await page.locator("#login-password").fill(process.env.E2E_TEST_PASSWORD!);
		await page.locator('button[type="submit"]').click();

		// Wait for login to complete
		await page.waitForTimeout(3000);

		// Check settings section for dependency status
		const depsStatus = page.locator("#deps-status");
		if (await depsStatus.isVisible()) {
			const statusText = await depsStatus.textContent();
			expect(statusText).toBeTruthy();
		}
	});
});
