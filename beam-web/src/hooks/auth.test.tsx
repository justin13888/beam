import { act, renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, describe, expect, it } from "vitest";
import { AuthProvider, useAuth } from "./auth";

const mockAuthResponse = {
	token: "test-jwt-token",
	session_id: "test-session-id",
	user: {
		id: "user-1",
		username: "testuser",
		email: "test@example.com",
		is_admin: false,
	},
};

function wrapper({ children }: { children: ReactNode }) {
	return <AuthProvider>{children}</AuthProvider>;
}

describe("useAuth", () => {
	afterEach(() => {
		localStorage.clear();
	});

	it("throws when used outside AuthProvider", () => {
		expect(() => renderHook(() => useAuth())).toThrow(
			"useAuth must be used within an AuthProvider",
		);
	});

	it("initial state: user is null and not authenticated", () => {
		const { result } = renderHook(() => useAuth(), { wrapper });
		expect(result.current.user).toBeNull();
		expect(result.current.token).toBeNull();
		expect(result.current.isAuthenticated).toBe(false);
	});

	it("after login(): user is set, isAuthenticated is true, localStorage updated", () => {
		const { result } = renderHook(() => useAuth(), { wrapper });

		act(() => {
			result.current.login(mockAuthResponse);
		});

		expect(result.current.user).toEqual(mockAuthResponse.user);
		expect(result.current.token).toBe(mockAuthResponse.token);
		expect(result.current.isAuthenticated).toBe(true);
		expect(localStorage.getItem("token")).toBe(mockAuthResponse.token);
		expect(JSON.parse(localStorage.getItem("user") ?? "null")).toEqual(
			mockAuthResponse.user,
		);
	});

	it("after logout(): user is null, isAuthenticated is false, localStorage cleared", () => {
		const { result } = renderHook(() => useAuth(), { wrapper });

		act(() => {
			result.current.login(mockAuthResponse);
		});

		act(() => {
			result.current.logout();
		});

		expect(result.current.user).toBeNull();
		expect(result.current.token).toBeNull();
		expect(result.current.isAuthenticated).toBe(false);
		expect(localStorage.getItem("token")).toBeNull();
		expect(localStorage.getItem("user")).toBeNull();
	});

	it("on mount with existing localStorage token: state is restored", async () => {
		localStorage.setItem("token", mockAuthResponse.token);
		localStorage.setItem("user", JSON.stringify(mockAuthResponse.user));

		const { result } = renderHook(() => useAuth(), { wrapper });

		// Wait for useEffect to restore state from localStorage
		await act(async () => {});

		expect(result.current.user).toEqual(mockAuthResponse.user);
		expect(result.current.token).toBe(mockAuthResponse.token);
		expect(result.current.isAuthenticated).toBe(true);
	});
});
