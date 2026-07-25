import { defineConfig } from "vite";
import { resolve } from "path";

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
});
