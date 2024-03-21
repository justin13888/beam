import { TypeOf, z } from "zod"

export const envSchema = z.object({
    NODE_ENV: z.string().default("development"),
    PORT: z.string().default("3000"),
    JWT_SECRET: z.string(),
    MYSQL_URL: z.string(),
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
