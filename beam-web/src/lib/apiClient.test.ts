import { describe, expect, it } from "vitest";
import { apiClient } from "./apiClient";

describe("apiClient", () => {
	it("is defined and not null", () => {
		expect(apiClient).toBeDefined();
		expect(apiClient).not.toBeNull();
	});

	it("has expected HTTP method functions", () => {
		expect(typeof apiClient.GET).toBe("function");
		expect(typeof apiClient.POST).toBe("function");
		expect(typeof apiClient.PUT).toBe("function");
		expect(typeof apiClient.DELETE).toBe("function");
	});
});
