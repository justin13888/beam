import { Elysia } from "elysia";
import { cors } from '@elysiajs/cors'
import { serverTiming } from '@elysiajs/server-timing'
import { swagger } from '@elysiajs/swagger'
import users from "./routes/user";
import { logger } from "./logger";
import { envs } from "./env";
import { documentation } from "./swagger";

// Start Elysia
const app = new Elysia()
  // .state('JWT_SECRET', envs.JWT_SECRET)
  .use(cors())
  .use(serverTiming())
  .use(swagger({
    documentation,
  }))
  .use(users)
  .get("/", () => "Hello from Beam")
  .listen(envs.PORT);

logger.info(`🦊 Elysia is running at ${app.server?.hostname}:${app.server?.port}`);
