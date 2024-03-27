import { Elysia } from "elysia";
import { logger } from "@/logger";
import register from "./register";
import profile from "./profile";
import logout from "./logout";
import login from "./login";
import elysiaLogger from "@/plugins/logger";
import session from "./session";
import { JWT_REFRESH_TOKEN_DURATION } from "@/lib/constants";
import { db } from "@/db/drizzle";
import { randomUUID } from "crypto";
import { sessions } from "@/db/drizzle/schema";

const user = new Elysia({ prefix: '/user' })
    .use(elysiaLogger)
    // TODO: Rate limit everything
    .use(profile)
    .use(register)
    .use(login)
    .use(logout)
    .use(session)
    // .get('/me', ({ bearer }) => bearer, { // TODO: Returns basic user info (username, email, name, etc.)
    // .post('/forgot-password', () => "") // TODO: Return consistent message with amount time. Send email with reset link. register URL token
    // .post('/reset-password', () => "") // TODO: Reset password with token (from email link)
    // .post('/password-change', () => "") // TODO: Change password should require RE-authentication and current password
    // .delete('/', () => "") // TODO: Delete account

export default user;

/**
 * Generate a session for a user
 * @param username Username
 * @param deviceName Device name
 * @param os Operating system
 * @param ip IP address
 * @returns Session ID
 */
export const generateSession = async (
    username: string,
    deviceName: string,
    os: 'windows' | 'mac' | 'linux' | 'android' | 'ios' | 'other',
    ip: string, // TODO: Require something not string. consider storing as enum of IPV4, IPV6, etc.
) => {
    const sessionId = randomUUID();
    const creationTime = Math.floor(Date.now() / 1000);
    const expiresAt = creationTime + JWT_REFRESH_TOKEN_DURATION;
    await db.insert(sessions).values({
        id: sessionId.toString(),
        createdAt: new Date(creationTime * 1000),
        expiresAt: new Date(expiresAt * 1000),
        lastUsedAt: new Date(creationTime * 1000),
        username,
        deviceName,
        os,
        ip,
        loginMethod: 'password',
    }).execute();

    return sessionId;
};
