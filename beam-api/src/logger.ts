import pino from "pino";
import { envs } from "./env";

// Initialize logger
export const initLogger = (environment: "development" | "production") => {
  // Development -> pretty print
  // Production -> asynchronous logging
  let logger: pino.Logger;
  if (environment === "development") {
    logger = pino({
      transport: {
        target: 'pino-pretty',
        options: {
          colorize: true
        }
      },
      level: 'debug',
    });
  } else {
    logger = pino(pino.destination({
      minLength: 4096, // Buffer before writing
      sync: false, // Asynchronous logging
      level: envs.PINO_LOG_LEVEL,
    }));
  }

  return logger;
};

const isDevelopment = envs.NODE_ENV === 'development';
export const logger = initLogger(isDevelopment ? "development" : "production");
if (isDevelopment) {
  logger.info("Running in development mode");
}
