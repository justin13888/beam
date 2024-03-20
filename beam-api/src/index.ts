import { Elysia } from "elysia";
import { cors } from '@elysiajs/cors'
import { serverTiming } from '@elysiajs/server-timing'
import { swagger } from '@elysiajs/swagger'
import users from "./users";
import pino from "pino";

// Parse environment variables
if (!process.env.JWT_SECRET) {
  throw new Error("Environment variable JWT_SECRET is required");
}
const JWT_SECRET = process.env.JWT_SECRET;

// Initialize logger
// Development -> pretty print
// Production -> asynchronous logging
let logger: pino.Logger;
if (process.env.NODE_ENV === "development") {
  console.log("Running in development mode")
  logger = pino({
    transport: {
      target: 'pino-pretty',
      options: {
        colorize: true
      }
    }
   });
} else {
  logger = pino(pino.destination({
    dest: './my-file', // omit for stdout
    minLength: 4096, // Buffer before writing
    sync: false // Asynchronous logging
  }));
}

// Start Elysia
const app = new Elysia()
  .state('JWT_SECRET', JWT_SECRET)
  .decorate('logger', logger)
  .use(cors())
  .use(serverTiming())
  .use(swagger({
    documentation: {
      tags: [
        { name: 'Auth', description: 'Authentication' },
        { name: 'User', description: 'User' },
        { name: 'Admin', description: 'Admin' },
      ]
    }
  }))
  .use(users)
  .get("/", () => "Hello from Beam")
  .listen(3000);

logger.info(`🦊 Elysia is running at ${app.server?.hostname}:${app.server?.port}`);
