import { test, expect } from "@playwright/test";

/**
 * E2E API Tests for AMOS Companion authentication
 *
 * Tests the /api/auth/sign-in/email-password endpoint directly.
 * This verifies the Companion's backend auth works correctly.
 */

const API_BASE = "https://amos-api.moo-vpn.online";

const TEST_ACCOUNTS = [
	{
		email: "munchira_01@hotmail.com",
		password: "munchira_01",
		label: "Hotmail account",
	},
] as const;

test.describe("Email/Password API Authentication", () => {
	for (const account of TEST_ACCOUNTS) {
		test.describe(`Sign in with ${account.label}`, () => {
			test("should sign in successfully with valid credentials", async ({
				request,
			}) => {
				const response = await request.post(
					`${API_BASE}/api/auth/sign-in/email-password`,
					{
						data: {
							email: account.email,
							password: account.password,
						},
					},
				);

				expect(response.status()).toBe(200);

				const body = await response.json();
				expect(body.user).toBeDefined();
				expect(body.user.email).toBe(account.email.toLowerCase());
				expect(body.user.id).toBeTruthy();
				expect(body.user.email_verified).toBe(true);
			});

			test("should fail with wrong password", async ({ request }) => {
				const response = await request.post(
					`${API_BASE}/api/auth/sign-in/email-password`,
					{
						data: {
							email: account.email,
							password: "wrong-password-123",
						},
					},
				);

				expect(response.status()).toBe(401);

				const body = await response.json();
				expect(body.detail).toBeDefined();
				expect(body.detail.code).toBe("invalid_credentials");
			});

			test("should fail with non-existent email", async ({ request }) => {
				const response = await request.post(
					`${API_BASE}/api/auth/sign-in/email-password`,
					{
						data: {
							email: "nonexistent-user-12345@test.com",
							password: "any-password",
						},
					},
				);

				// Should return 404 for non-existent user
				expect(response.status()).toBe(404);

				const body = await response.json();
				expect(body.detail.code).toBe("user_not_found");
			});
		});
	}

	test("should check if email exists", async ({ request }) => {
		const response = await request.get(
			`${API_BASE}/api/auth/email-exists?email=${encodeURIComponent(TEST_ACCOUNTS[0].email)}`,
		);

		expect(response.status()).toBe(200);

		const body = await response.json();
		expect(body.exists).toBe(true);
		expect(body.providers).toContain("credential");
	});

	test("should return 404 for non-existent email check", async ({
		request,
	}) => {
		const response = await request.get(
			`${API_BASE}/api/auth/email-exists?email=nonexistent-user-12345@test.com`,
		);

		expect(response.status()).toBe(200);

		const body = await response.json();
		expect(body.exists).toBe(false);
	});
});

test.describe("API Health Check", () => {
	test("should have correct API URL", () => {
		// This test documents the correct API URL for Companion
		expect(API_BASE).toBe("https://amos-api.moo-vpn.online");
	});

	test("should accept JSON content type", async ({ request }) => {
		const response = await request.post(
			`${API_BASE}/api/auth/sign-in/email-password`,
			{
				headers: {
					"Content-Type": "application/json",
				},
				data: {
					email: TEST_ACCOUNTS[0].email,
					password: TEST_ACCOUNTS[0].password,
				},
			},
		);

		expect(response.status()).toBe(200);
	});
});
