import { fileURLToPath, URL } from "node:url";
import viteReact from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
	plugins: [viteReact()],
	resolve: {
		alias: {
			"@": fileURLToPath(new URL("./src", import.meta.url)),
		},
	},
	test: {
		environment: "jsdom",
		globals: true,
		setupFiles: ["./src/test/setup.ts"],
		passWithNoTests: true,
		env: {
			C_STREAM_SERVER_URL: "http://localhost:8000",
		},
		coverage: {
			provider: "v8",
			reporter: ["text", "lcov", "html"],
			reportsDirectory: "coverage",
			// TODO: Enforce 80% thresholds once test coverage reaches that level:
			// thresholds: {
			//   lines: 80,
			//   functions: 80,
			//   branches: 80,
			//   statements: 80,
			// },
		},
	},
});
