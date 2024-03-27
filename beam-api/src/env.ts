import { TypeOf, z } from "zod"

export const envSchema = z.object({
    NODE_ENV: z.string().default("development"),
    PORT: z.string().default("3000"),
    PINO_LOG_LEVEL: z.string().default("info"),
    /**
     * File path to JWT keys
     */
    JWT_SECRET_PATH: z.string(),
    /**
     * URL of the PostgresURL server
     */
    POSTGRES_URL: z.string(),
    /**
     * URL of the Redis server
     */
    // REDIS_URL: z.string(),
    /**
     * Random pepper for Argon2id
     */
    ARGON2ID_PEPPER: z.string().min(128),
    /**
     * Memory cost of Argon2id in KiB
     */
    ARGON2ID_MEMORY_COST: z.number().min(2 ** 16) .max(2 ** 32).default(2 ** 16),
    /**
     * Time cost of Argon2id, measured in number of iterations
     */
    ARGON2ID_TIME_COST: z.number().min(1).max(10).default(3),
})

// declare global {
//     namespace NodeJS {
//         interface ProcessEnv extends TypeOf<typeof envSchema> { }
//     }
// }

const getEnvs = () => {
    try {
        return envSchema.parse(process.env);
    } catch (err) {
        if (err instanceof z.ZodError) {
            const { fieldErrors } = err.flatten()
            const errorMessage = Object.entries(fieldErrors)
                .map(([field, errors]) =>
                    errors ? `${field}: ${errors.join(", ")}` : field,
                )
                .join("\n  ")
            throw new Error(
                `Missing environment variables:\n  ${errorMessage}`,
            )
        } else {
            throw err;
        }
    }
};

export const envs = getEnvs();
