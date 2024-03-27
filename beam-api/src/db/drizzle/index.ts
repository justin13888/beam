import { drizzle } from 'drizzle-orm/postgres-js'
import postgres from 'postgres'
import { envs } from "@/env";
import * as schema from "./schema";

// Disable prefetch as it is not supported for "Transaction" pool mode 
const client = postgres(envs.POSTGRES_URL, { prepare: false });
export const db = drizzle(client, { schema });
