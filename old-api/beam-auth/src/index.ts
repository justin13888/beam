import { cors } from "@elysiajs/cors";
import { serverTiming } from "@elysiajs/server-timing";
import { swagger } from "@elysiajs/swagger";
import { Elysia } from "elysia";
import { envs } from "./env";
import { logger } from "./logger";
import token from "./routes/token";
import users from "./routes/user";
import { documentation } from "./swagger";

// Start Elysia
const app = new Elysia()
    .use(cors())
    .use(serverTiming())
    .use(
        swagger({
            documentation,
        }),
    )
    .use(users)
    .use(token)
    .get("/", () => "Hello from Beam")
    .listen(envs.PORT);

logger.info(
    `🦊 Elysia is running at ${app.server?.hostname}:${app.server?.port}`,
);
