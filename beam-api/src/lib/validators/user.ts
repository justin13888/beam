import { t } from "elysia";
import isEmail from "validator/es/lib/isEmail";
import {
    PASSWORD_MAX_LENGTH,
    PASSWORD_MIN_LENGTH,
    USERNAME_MAX_LENGTH,
    USERNAME_MIN_LENGTH,
} from "../constants";

/**
 * Validates email using isEmail from `validator`
 * @param email
 * @returns
 */
export const validateEmail = (email: string): boolean => {
    return isEmail(email);
};

/**
 * Normalizes username
 * @param username
 * @returns Normalized username
 */
export const normalizeUsername = (username: string): string => {
    return username.toLowerCase().trim();
};

/**
 * Validates username
 * @param username
 * @throws Error if username is invalid
 */
export const validateUsername = (username: string): void => {
    const minLength = USERNAME_MIN_LENGTH;
    const maxLength = USERNAME_MAX_LENGTH;

    // Minimum length
    if (username.length < minLength) {
        throw new Error(`Username must be at least ${minLength} character`);
    }
    // Maximum length
    if (username.length > maxLength) {
        throw new Error(`Username must be at most ${maxLength} characters`);
    }

    // Contains only alphanumeric, underscore, hyphen, period, number, no whitespace
    if (!/^[a-zA-Z0-9_.-]+$/.test(username)) {
        throw new Error(
            "Username can only contain alphanumeric characters, underscore, hyphen, period, and no whitespace",
        );
    }

    // Is not reserved
    const lowercaseUsername = username.toLowerCase();
    if (
        lowercaseUsername === "admin" ||
        lowercaseUsername === "administrator"
    ) {
        throw new Error("Username is reserved");
    }
};

// Validate password rules
// Throws error if password is invalid
export const validatePassword = (password: string): void => {
    const minLength = PASSWORD_MIN_LENGTH;
    const maxLength = PASSWORD_MAX_LENGTH;
    // Minimum length
    if (password.length < minLength) {
        throw new Error(
            `Password (${password.length}) must be at least ${minLength} character(s)`,
        );
    }
    // Maximum length
    if (password.length > maxLength) {
        throw new Error(
            `Password (${password.length}) must be at most ${maxLength} characters`,
        );
    }
};
