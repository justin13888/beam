import { logger } from "@/logger";
import elysiaLogger from "@/plugins/logger";
import Elysia from "elysia";

// TODO: Consider streaming
// <https://elysiajs.com/plugins/stream.html>
const collection = new Elysia({ prefix: "/collection" })
    .use(elysiaLogger)
    // .post('/', () => "") // TODO: Create collection
    // .get('/', () => "") // TODO: Get collections
    // .get('/:id', () => "") // TODO: Get collection
    // .put('/:id', () => "") // TODO: Update collection
    // .delete('/:id', () => "") // TODO: Delete collection
    .get(
        "/search",
        () => {
            logger.info("Searching for collections");
            return "Searching for collections";
        },
        {
            detail: {
                tags: ["Collection"],
                summary: "Search collections",
                description: "Search collections",
            },
        },
    );

// TODO: Add the rest

export default collection;
