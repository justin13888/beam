import { randomUUID } from "node:crypto";
import { db } from "@/db/drizzle";
import { users } from "@/db/drizzle/schema";
import {
    type ScopesType,
    jwtAccessSetup,
    jwtRefreshSetup,
} from "@/plugins/auth";
import { Scopes, defaultScopes } from "@/plugins/auth";
import elysiaLogger from "@/plugins/logger";
import { bearer } from "@elysiajs/bearer";
import type { JWTPayloadSpec } from "@elysiajs/jwt";
import { Elysia, t } from "elysia";

// TODO: Implement throttle for how frequent tokens can be refreshed
const token = new Elysia({ prefix: "/token" })
    .use(elysiaLogger)
    .use(jwtAccessSetup)
    .use(jwtRefreshSetup)
    .use(bearer())
    .post(
        "/refresh",
        async ({ jwtAccess, jwtRefresh, set, bearer, body }) => {
            const scopes: ScopesType = {
                ...defaultScopes,
                ...body.scopes,
            };
            const user = await jwtRefresh.verify(bearer);
            console.log(user);
            if (!user) {
                set.status = 401;
                return "Unauthorized";
            }

            // Validate scopes
            try {
                validateScopes(user, scopes);
            } catch (error) {
                set.status = 401;
                if (error instanceof Error) {
                    return error.message;
                }
                return "Invalid scopes";
            }

            const signedJWT = await jwtAccess.sign({
                id: randomUUID(),
                scopes: scopes,
            });
            console.log(signedJWT);
            console.log(await jwtAccess.verify(signedJWT));

            return signedJWT;
        },
        {
            body: t.Object({
                scopes: t.Partial(Scopes),
            }),
            response: t.String(),
            detail: {
                tags: ["Auth"],
                summary: "Refresh access token",
                description: "Obtain a new access token using a refresh token",
            },
            error({ error }) {
                return {
                    status: 401,
                    body: error.message,
                };
            },
        },
    );

export default token;

const validateScopes = async (
    refreshToken: { id: string } & JWTPayloadSpec,
    scopes: ScopesType,
) => {
    // Only sudo requires a refresh token
    if (scopes.sudo) {
        // Check if user has admin role
        const user = await db.query.users
            .findFirst({
                columns: {
                    isAdmin: true,
                },
                where: (user, { eq }) => eq(user.username, refreshToken.id),
            })
            .execute();
        const isAdmin = user?.isAdmin ?? false;

        if (!isAdmin) {
            throw new Error("sudo scope requires admin role");
        }
    }

    // Some roles require recent refresh token
    if (scopes["account:modify"] || scopes["account:delete"]) {
        // Refresh token must be less than 5 minutes old
        if (
            !refreshToken.iat ||
            refreshToken.iat < Math.floor(Date.now() / 1000) - 5 * 60
        ) {
            throw new Error(
                "Account modification requires recent refresh token",
            );
        }
    }
};
// TODO: Add endpoint for re-authentication
