export const DATABASE_PREFIX = "beam"; // E.g. if prefix is "beam", then a users table name might be "beam_users"

export const USERNAME_MIN_LENGTH = 3;
export const USERNAME_MAX_LENGTH = 32;
export const PASSWORD_MIN_LENGTH = 8;
export const PASSWORD_MAX_LENGTH = 128;

export const JWT_ALGORITHM = "EdDSA";
/**
 * The duration of the refresh token in seconds.
 */
export const JWT_REFRESH_TOKEN_DURATION = 30 * 24 * 60 * 60; // 30 days
export const JWT_ACCESS_TOKEN_DURATION = 15 * 60; // 15 minutes
export const JWT_ISSUER = "urn:beam:api";
