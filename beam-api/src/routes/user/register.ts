import {
    normalizeUsername,
    validateEmail,
    validatePassword,
    validateUsername,
} from "@/lib/validators/user";
import { normalizeEmail } from "validator";

import { db } from "@/db/drizzle/index";
import { users } from "@/db/drizzle/schema";
import { hashPassword } from "@/lib/hash";
import { refreshTokenModel } from "@/models/token";
import { newUserModel } from "@/models/user";
import { jwtAccessSetup, jwtRefreshSetup } from "@/plugins/auth";
import { eq } from "drizzle-orm";
import Elysia, { ParseError, ValidationError, t } from "elysia";
import { generateSession } from ".";

const register = new Elysia()
    .use(newUserModel)
    .use(jwtRefreshSetup)
    .use(refreshTokenModel)
    .post(
        "/register",
        async ({ body, jwtRefresh }) => {
            const email = body.email;
            const username = body.username;
            const password = body.password;

            // Validate email satisfies rules
            if (!validateEmail(email)) {
                throw new Error("Invalid email");
            }

            // Validate username satisfies rules
            try {
                validateUsername(username);
            } catch (error) {
                if (error instanceof Error) {
                    throw new ParseError(`Invalid username: ${error.message}`);
                }

                throw new Error("Invalid username: Unknown error");
            }

            // Validate password satisfies rules
            try {
                validatePassword(password);
            } catch (error) {
                if (error instanceof Error) {
                    throw new ParseError(`Invalid password: ${error.message}`);
                }
                throw new Error("Invalid password: Unknown error");
            }

            // Normalize email
            const normalizedEmail = normalizeEmail(email);
            if (!normalizedEmail) {
                throw new ParseError(
                    "Invalid email: Failed to normalize email",
                );
            }
            const normalizedUsername = normalizeUsername(username);

            // Validate email is not already in use
            const emailExists = await db
                .select()
                .from(users)
                .where(eq(users.email, normalizedEmail))
                .execute();
            if (emailExists.length > 0) {
                throw new ParseError("Email is already in use");
            }

            // Validate username is not already in use
            const existingUser = await db
                .select()
                .from(users)
                .where(eq(users.username, normalizedUsername))
                .execute();
            if (existingUser.length > 0) {
                throw new ParseError("Username is already in use");
            }

            // Hash password
            const hashedPassword = await hashPassword(password);

            // Create user with email, username, password
            await db
                .insert(users)
                .values({
                    email: normalizedEmail,
                    username: normalizedUsername,
                    hashedPassword: hashedPassword,
                })
                .execute();

            const deviceName = "Unknown"; // TODO: Ask from user
            const os = "other"; // TODO: Get from user agent
            // console.log(ip);

            const sessionId = await generateSession(
                normalizedUsername,
                deviceName,
                os,
                "1", // TODO: Get IP
            );

            const signedJWT = await jwtRefresh.sign({
                id: sessionId,
            });

            // console.log(signedJWT);
            return {
                refresh_token: signedJWT,
            };
        },
        {
            body: "newUser",
            response: "refreshToken",
            detail: {
                tags: ["Auth"],
                summary: "Register",
                description: "Register to get user token",
            },
            error({ error }) {
                if (error instanceof ParseError) {
                    return {
                        status: 400,
                        body: {
                            message: error.message,
                        },
                    };
                }

                return {
                    status: 500,
                };
            },
        },
    );

// TODO: Prevent password reuse
// TODO: Mark new users so onboardings can be triggered.

export default register;
