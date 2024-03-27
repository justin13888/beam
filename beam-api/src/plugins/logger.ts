import { logger } from "@/logger";
import Elysia from "elysia";

const elysiaLogger = new Elysia()
    .decorate('logger', logger)

export default elysiaLogger;
