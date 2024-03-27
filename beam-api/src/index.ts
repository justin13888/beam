import { Elysia } from "elysia";
import { cors } from '@elysiajs/cors'
import { serverTiming } from '@elysiajs/server-timing'
import { swagger } from '@elysiajs/swagger'
import users from "./routes/user";
import { logger } from "./logger";
import { envs } from "./env";
import { documentation } from "./swagger";
import token from "./routes/token";

// Start Elysia
const app = new Elysia()
  .use(cors())
  .use(serverTiming())
  .use(swagger({
    documentation,
  }))
  .use(users)
  .use(token)
  .get("/", () => "Hello from Beam")
  .listen(envs.PORT);

logger.info(`🦊 Elysia is running at ${app.server?.hostname}:${app.server?.port}`);
