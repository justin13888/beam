import "@testing-library/jest-dom/vitest";
import { afterAll, afterEach, beforeAll, vi } from "vitest";
import { server } from "./server";

// Bun passes `--localstorage-file` without a valid path in certain environments
// (e.g. a Bazel action with a sanitised env).  When this happens bun injects a
// plain `{}` object instead of a proper Web Storage implementation, and jsdom's
// localStorage cannot override it.  Detect this and replace it with a
// spec-conformant in-memory stub so tests run reliably in all environments.
if (
	typeof localStorage === "undefined" ||
	typeof localStorage.getItem !== "function"
) {
	const store = new Map<string, string>();
	vi.stubGlobal("localStorage", {
		getItem: (key: string) => store.get(key) ?? null,
		setItem: (key: string, value: string) => {
			store.set(key, value);
		},
		removeItem: (key: string) => {
			store.delete(key);
		},
		clear: () => {
			store.clear();
		},
		key: (index: number) => Array.from(store.keys())[index] ?? null,
		get length() {
			return store.size;
		},
	});
}

beforeAll(() => server.listen({ onUnhandledRequest: "error" }));
afterEach(() => {
	server.resetHandlers();
	localStorage.clear();
});
afterAll(() => server.close());
