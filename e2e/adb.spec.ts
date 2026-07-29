import { test, expect, describe } from "@playwright/test";

/**
 * E2E Tests for ADB Device Management
 *
 * These tests verify ADB functionality on macOS/Linux with a connected device.
 * Requires: adb installed and device connected via USB.
 *
 * Run with: npx playwright test e2e/adb.spec.ts
 */

// ADB test configuration
const ADB_TIMEOUT = 10000;
const DEVICE_SERIAL = process.env.ADB_DEVICE_SERIAL || "R7AL40JZNMV"; // Replace with your device

describe.serial("ADB Device Connection", () => {
	test("should detect connected device", async () => {
		const { exec } = await import("child_process");
		const adb = await new Promise<string>((resolve, reject) => {
			exec("which adb || echo '/usr/local/bin/adb'", (err, stdout) => {
				if (err) reject(err);
				else resolve(stdout.trim());
			});
		});

		expect(adb).toBeTruthy();

		const devices = await new Promise<string>((resolve, reject) => {
			exec(`${adb} devices -l`, { timeout: ADB_TIMEOUT }, (err, stdout) => {
				if (err) reject(err);
				else resolve(stdout);
			});
		});

		console.log("ADB devices output:", devices);
		expect(devices).toContain("device");
	});

	test("should get device properties", async () => {
		const { exec } = await import("child_process");
		const adb = "/usr/local/bin/adb";

		const model = await new Promise<string>((resolve, reject) => {
			exec(`${adb} -s ${DEVICE_SERIAL} shell getprop ro.product.model`, { timeout: ADB_TIMEOUT }, (err, stdout) => {
				if (err) reject(err);
				else resolve(stdout.trim());
			});
		});

		console.log("Device model:", model);
		expect(model).toBeTruthy();
	});

	test("should list installed packages", async () => {
		const { exec } = await import("child_process");
		const adb = "/usr/local/bin/adb";

		const packages = await new Promise<string>((resolve, reject) => {
			exec(`${adb} -s ${DEVICE_SERIAL} shell pm list packages`, { timeout: ADB_TIMEOUT }, (err, stdout) => {
				if (err) reject(err);
				else resolve(stdout);
			});
		});

		console.log("Packages count:", packages.split("\n").length);
		expect(packages).toContain("package:");
	});

	test("should check screen dimensions", async () => {
		const { exec } = await import("child_process");
		const adb = "/usr/local/bin/adb";

		const dims = await new Promise<string>((resolve, reject) => {
			exec(`${adb} -s ${DEVICE_SERIAL} shell wm size`, { timeout: ADB_TIMEOUT }, (err, stdout) => {
				if (err) reject(err);
				else resolve(stdout.trim());
			});
		});

		console.log("Screen dimensions:", dims);
		expect(dims).toMatch(/Physical size:|Override size:/);
	});
});

describe.serial("ADB Device Control", () => {
	test("should unlock device screen", async () => {
		const { exec } = await import("child_process");
		const adb = "/usr/local/bin/adb";

		// Wake up the device
		await new Promise<void>((resolve, reject) => {
			exec(`${adb} -s ${DEVICE_SERIAL} shell input keyevent KEYCODE_WAKEUP`, { timeout: ADB_TIMEOUT }, (err) => {
				if (err) console.warn("Wake failed:", err.message);
				resolve();
			});
		});

		// Unlock with swipe up (assuming lock screen)
		await new Promise<void>((resolve, reject) => {
			exec(`${adb} -s ${DEVICE_SERIAL} shell input swipe 360 1000 360 200`, { timeout: ADB_TIMEOUT }, (err) => {
				if (err) console.warn("Swipe failed:", err.message);
				resolve();
			});
		});

		console.log("Device wake/unlock attempted");
	});

	test("should perform tap action", async () => {
		const { exec } = await import("child_process");
		const adb = "/usr/local/bin/adb";

		await new Promise<void>((resolve, reject) => {
			exec(`${adb} -s ${DEVICE_SERIAL} shell input tap 360 600`, { timeout: ADB_TIMEOUT }, (err) => {
				if (err) reject(err);
				else resolve();
			});
		});

		console.log("Tap performed at 360,600");
	});

	test("should perform swipe action", async () => {
		const { exec } = await import("child_process");
		const adb = "/usr/local/bin/adb";

		await new Promise<void>((resolve, reject) => {
			exec(`${adb} -s ${DEVICE_SERIAL} shell input swipe 360 1000 360 200 500`, { timeout: ADB_TIMEOUT }, (err) => {
				if (err) reject(err);
				else resolve();
			});
		});

		console.log("Swipe performed");
	});

	test("should send text input", async () => {
		const { exec } = await import("child_process");
		const adb = "/usr/local/bin/adb";

		// Test with a simple text
		await new Promise<void>((resolve, reject) => {
			exec(`${adb} -s ${DEVICE_SERIAL} shell input text hello`, { timeout: ADB_TIMEOUT }, (err) => {
				if (err) reject(err);
				else resolve();
			});
		});

		console.log("Text 'hello' sent");
	});

	test("should press back button", async () => {
		const { exec } = await import("child_process");
		const adb = "/usr/local/bin/adb";

		await new Promise<void>((resolve, reject) => {
			exec(`${adb} -s ${DEVICE_SERIAL} shell input keyevent KEYCODE_BACK`, { timeout: ADB_TIMEOUT }, (err) => {
				if (err) reject(err);
				else resolve();
			});
		});

		console.log("Back button pressed");
	});

	test("should press home button", async () => {
		const { exec } = await import("child_process");
		const adb = "/usr/local/bin/adb";

		await new Promise<void>((resolve, reject) => {
			exec(`${adb} -s ${DEVICE_SERIAL} shell input keyevent KEYCODE_HOME`, { timeout: ADB_TIMEOUT }, (err) => {
				if (err) reject(err);
				else resolve();
			});
		});

		console.log("Home button pressed");
	});
});

describe.serial("ADB Screenshot & Screenrecord", () => {
	test("should capture screenshot", async () => {
		const { exec } = await import("child_process");
		const fs = await import("fs");
		const adb = "/usr/local/bin/adb";

		const localPath = "/tmp/test-screenshot.png";
		const devicePath = "/sdcard/test-screenshot.png";

		// Take screenshot on device
		await new Promise<void>((resolve, reject) => {
			exec(`${adb} -s ${DEVICE_SERIAL} shell screencap ${devicePath}`, { timeout: ADB_TIMEOUT }, (err) => {
				if (err) reject(err);
				else resolve();
			});
		});

		// Pull to local
		await new Promise<void>((resolve, reject) => {
			exec(`${adb} -s ${DEVICE_SERIAL} pull ${devicePath} ${localPath}`, { timeout: ADB_TIMEOUT }, (err) => {
				if (err) reject(err);
				else resolve();
			});
		});

		// Verify file exists
		const exists = fs.existsSync(localPath);
		console.log("Screenshot saved:", localPath, exists);

		// Cleanup
		try { fs.unlinkSync(localPath); } catch {}
		await new Promise<void>((resolve) => {
			exec(`${adb} -s ${DEVICE_SERIAL} shell rm ${devicePath}`, () => resolve());
		});

		expect(exists).toBe(true);
	});

	test("should check screenrecord capability", async () => {
		const { exec } = await import("child_process");
		const adb = "/usr/local/bin/adb";

		// Check if screenrecord is available
		const result = await new Promise<string>((resolve, reject) => {
			exec(`${adb} -s ${DEVICE_SERIAL} shell screenrecord --help`, { timeout: ADB_TIMEOUT }, (err, stdout) => {
				if (err) resolve("");
				else resolve(stdout);
			});
		});

		console.log("Screenrecord available:", result.includes("Usage:"));
		// Note: screenrecord is available on Android 4.4+
	});
});
