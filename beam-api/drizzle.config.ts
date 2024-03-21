import { type Config } from "drizzle-kit";
import { envs } from "@/env";
import { DATABASE_PREFIX } from "@/lib/constants";

export default {
    schema: "src/db/schema.ts",
    out: "./drizzle",
    driver: "mysql2",
    dbCredentials: {
        uri: envs.MYSQL_URL,
    },
    tablesFilter: [`${DATABASE_PREFIX}_*`],
} satisfies Config;
