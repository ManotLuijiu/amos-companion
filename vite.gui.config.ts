import { readFileSync } from "fs";
import { resolve } from "path";
import { defineConfig } from "vite";

const packageJson = JSON.parse(
	readFileSync(resolve(__dirname, "package.json"), "utf8"),
) as { version: string };

// Vite config for the Tauri GUI (src-ui/)
export default defineConfig({
	root: "src-ui",
	base: "./",
	build: {
		outDir: "../dist",
		emptyOutDir: true,
		rollupOptions: {
			input: {
				main: resolve(__dirname, "src-ui/index.html"),
			},
		},
		target: "esnext",
		minify: "esbuild",
	},
	server: {
		port: 1420,
		strictPort: true,
	},
	envPrefix: ["VITE_", "TAURI_"],
	define: {
		__APP_VERSION__: JSON.stringify(packageJson.version),
	},
});
