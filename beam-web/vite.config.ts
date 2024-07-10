import { resolve } from "node:path";
import { TanStackRouterVite } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react-swc";
import { defineConfig } from "vite";
import { ViteMinifyPlugin } from "vite-plugin-minify";

/** @type {import('vite').UserConfig} */
export default defineConfig({
  build: {
    rollupOptions: {
      input: {
        file1: resolve(__dirname, "index.html"),
      },
    },
  },
  plugins: [ViteMinifyPlugin({}), TanStackRouterVite(), react()],
});
// TODO: Configure PWA
