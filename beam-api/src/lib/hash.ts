import { envs } from "@/env";
import * as argon2 from "argon2";
/**
 * Hash password with argon2id
 * @param password Password to hash
 * @returns Hashed password
 */
export const hashPassword = async (password: string) => {
    return argon2.hash(password, {
        type: argon2.argon2id,
        memoryCost: envs.ARGON2ID_MEMORY_COST,
        timeCost: envs.ARGON2ID_TIME_COST,
        secret: Buffer.from(envs.ARGON2ID_PEPPER),
    });
    // return Bun.password.hash(password, {
    //     algorithm: "argon2id",
    //     memoryCost: envs.ARGON2ID_MEMORY_COST,
    //     timeCost: envs.ARGON2ID_TIME_COST,
    //     // secret: Buffer.from(envs.ARGON2ID_PEPPER),
    // });
    // TODO: Replace with bun if they ever support peppers: https://bun.sh/docs/api/hashing
};
// TODO: Check timeCost takes around 500ms-1000ms

/**
 * Verify password with argon2id
 * @param digest Hashed password
 * @param password Password to verify
 * @returns `true` if the digest parameters matches the hash generated from `plain`, otherwise `false`
 */
export const verifyPassword = async (digest: string, password: Buffer | string) => {
    return argon2.verify(digest, password, {
        secret: Buffer.from(envs.ARGON2ID_PEPPER),
    });
};
