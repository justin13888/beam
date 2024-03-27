import { db } from "@/db/drizzle";
import { sessions } from "@/db/drizzle/schema";
import { jwtRefreshSetup } from "@/plugins/auth";
import elysiaLogger from "@/plugins/logger";
import bearer from "@elysiajs/bearer";
import { eq } from "drizzle-orm";
import Elysia from "elysia";

const logout = new Elysia()
    .use(elysiaLogger)
    .use(jwtRefreshSetup)
    .use(bearer())
    // TODO: Add rate limiting
    .post(
        "/logout",
        async ({ bearer, jwtRefresh }) => {
            const refreshToken = await jwtRefresh.verify(bearer);
            if (!refreshToken) {
                return "Unauthorized";
            }

            // Look for refresh token and mark it as invalid
            // Then move it elsewhere
            const result = await db
                .update(sessions)
                .set({ revoked: true })
                .where(eq(sessions.id, refreshToken.id));
        },
        {
            detail: {
                tags: ["User"],
                summary: "Logout",
                description: "Logout of the current session",
            },
            response: {
                status: 200,
                body: "OK",
            },
            error({ error }) {
                return {
                    status: 401,
                    body: error.message,
                };
            },
        },
    ); // TODO: Support logging out all devices, logout other user devices

export default logout;
