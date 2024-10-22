import { envs } from "@/env";
import { DATABASE_PREFIX } from "@/lib/constants";
import type { Config } from "drizzle-kit";

export default {
    schema: "src/db/drizzle/schema.ts",
    out: "./drizzle",
    driver: "pg",
    dbCredentials: {
        connectionString: envs.POSTGRES_URL,
    },
    tablesFilter: [`${DATABASE_PREFIX}_*`],
} satisfies Config;
