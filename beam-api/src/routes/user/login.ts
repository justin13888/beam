import { db } from "@/db/drizzle";
import { loginHistory, sessions, users } from "@/db/drizzle/schema";
import { verifyPassword } from "@/lib/hash";
import { normalizeUsername } from "@/lib/validators/user";
import { userCredentialsModel } from "@/models/user";
import { jwtAccessSetup, jwtRefreshSetup } from "@/plugins/auth";
import elysiaLogger from "@/plugins/logger";
import { eq } from "drizzle-orm";
import Elysia, { InternalServerError, NotFoundError, ParseError, ValidationError, t } from "elysia";
import { normalizeEmail } from "validator";
// import { ip } from "elysia-ip"; // TODO: library is buggy
import { generateSession } from ".";
import { randomUUID } from "crypto";
import { refreshTokenModel } from "@/models/token";
import { rateLimit } from 'elysia-rate-limit'

/**
 * Returns refresh token if username and password are valid
 * @param username Username or email
 * @param password Password
 * @returns Refresh token
 * @throws Error if username/email or password is invalid
 */
const login = new Elysia()
    .use(elysiaLogger)
    .use(userCredentialsModel)
    .use(refreshTokenModel)
    .use(jwtRefreshSetup)
    // .use(ip())
    .use(rateLimit({ // TODO: Implement login throttle (double exponential with 1 second base)
        max: 1,
        duration: 60,
        responseMessage: "Too many login attempts. Please try again later.",
    }))
    .onError(({ code, error, logger }) => {
        if (error instanceof CredentialsError) {
            logger.debug(`Credentials error occurred: ${error.message}`);
        } else if (error instanceof ValidationError) {
            logger.debug(`Validation error occurred: ${error.message}`);
        } else if (error instanceof NotFoundError) {
            logger.debug(`Not found error occurred: ${error.message}`);
        } else {
            logger.error(`Error code ${code} occurred: ${error.message}`);
        }
    })
    .post('/login', async ({ body, jwtRefresh, logger }) => {
        const username = body.username;
        const password = body.password;

        let user;
        try {
            user = await getUserByUsernameOrEmail(username);
            if (!user) {
                throw new NotFoundError("User does not exist");
            }
        } catch (error) {
            let errorMsg = "Unknown";
            if (error instanceof NotFoundError) {
                errorMsg = error.message;
            }
            logger.debug(`Error occurred while trying to find user: ${errorMsg}`);
            throw error;
        }

        // Verify password
        let passwordMatch = false; // Default value assuming no match
        try {
            passwordMatch = await verifyPassword(user.hashedPassword, password);
        } catch (error) {
            const errorMsg = error instanceof Error ? error.message : "Unknown";
            throw new InternalServerError(`Error occurred while trying to verify password: ${errorMsg}`);
        }

        const deviceName = 'Unknown'; // TODO: Ask from user
        const os = 'other'; // TODO: Get from user agent
        // console.log(ip);

        if (!passwordMatch) {
            // Log failed login attempt
            const l = await db.insert(loginHistory).values({
                username: user.username,
                deviceName,
                os,
                ip: '1',
            }).execute();

            throw new ParseError("Invalid username or password")
        }
        // console.log(ip);
        const sessionId = await generateSession(
            user.username,
            deviceName,
            os,
            '1', // TODO: Get IP
        );

        const signedJWT = await jwtRefresh.sign({
            id: sessionId,
        });

        // console.log(signedJWT);
        return {
            refresh_token: signedJWT,
        };
    }, {
        body: 'userCredentials',
        response: 'refreshToken',
        detail: {
            tags: ['Auth'],
            summary: 'Login',
            description: 'Login to get refresh token'
        },
        error({ error }) {
            if (error instanceof CredentialsError) {
                return {
                    status: 401,
                    body: {
                        message: error.message
                    }
                }
            } else if (error instanceof ValidationError) {
                return {
                    status: 400,
                    body: {
                        message: error.message
                    }
                }
            } else if (error instanceof InternalServerError) {
                return {
                    status: 500,
                    message: "Unable to login. Please try again later."
                }
            } else {
                return {
                    status: 500,
                }
            }
        }
    });

export default login;

// TODO: Implement 2FA (update schema, return with request for 2FA token)
// TODO: Besides 2fa via OTP, allow backup codes
// TODO: Implement account lockout policy
//   - Throttle login attempts
//   - Require CAPTCHA after 3 failed attempts
// TODO: If previously logged in, allow 2fa only authentication (skip credentials). Require 2fa again if it's been more than 30 days since last login
// TODO: Support webauthn: <https://github.com/MasterKale/SimpleWebAuthn?tab=readme-ov-file>

const getUserByUsernameOrEmail = async (usernameOrEmail: string) => {
    if (usernameOrEmail.includes('@')) {
        // Consider username as email
        const normalizedEmail = normalizeEmail(usernameOrEmail);

        if (!normalizedEmail) {
            throw new CredentialsError("Invalid email. If you mean to enter a username, please remove the @ symbol");
        }

        // Check if user exists
        return db.query.users.findFirst({
            where: eq(users.email, normalizedEmail)
        });
    } else {
        // Consider username as username

        const normalizedUsername = normalizeUsername(usernameOrEmail);

        // Check if user exists
        return db.query.users.findFirst({
            where: eq(users.username, normalizedUsername)
        });
    }
}

class CredentialsError extends Error {
    constructor(message: string) {
        super(message);
        this.name = 'CredentialsError';
    }
}
