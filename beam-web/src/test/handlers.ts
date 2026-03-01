import { HttpResponse, http } from "msw";

const BASE_URL = "http://localhost:8000";

const mockUser = {
	id: "user-1",
	username: "testuser",
	email: "test@example.com",
	is_admin: false,
};

export const mockAuthSuccess = {
	token: "test-jwt-token",
	session_id: "test-session-id",
	user: mockUser,
};

export const handlers = [
	// Default: successful login
	http.post(`${BASE_URL}/v1/auth/login`, () => {
		return HttpResponse.json(mockAuthSuccess);
	}),

	http.post(`${BASE_URL}/v1/auth/logout`, () => {
		return new HttpResponse(null, { status: 200 });
	}),

	http.post(`${BASE_URL}/v1/auth/register`, () => {
		return HttpResponse.json({
			token: "test-jwt-token",
			session_id: "test-session-id",
			user: {
				id: "user-2",
				username: "newuser",
				email: "new@example.com",
				is_admin: false,
			},
		});
	}),
];

// Reusable handler for a failed login (401)
export const loginFailureHandler = http.post(
	`${BASE_URL}/v1/auth/login`,
	() => {
		return HttpResponse.json(
			{ message: "Invalid credentials", code: "invalid_credentials" },
			{ status: 401 },
		);
	},
);
