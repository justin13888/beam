import { Elysia } from "elysia";
import { cors } from '@elysiajs/cors'
import { bearer } from '@elysiajs/bearer'
import { serverTiming } from '@elysiajs/server-timing'
import { swagger } from '@elysiajs/swagger'

const app = new Elysia()
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

  .use(bearer())
  .get('/sign', ({ bearer }) => bearer, {
    beforeHandle({ bearer, set }) {
      if (!bearer) {
        set.status = 400
        set.headers[
          'WWW-Authenticate'
        ] = `Bearer realm='sign', error="invalid_request"`

        return 'Unauthorized'
      }
    }
  })
  .get("/", () => "Hello Elysia").listen(3000);

console.log(
  `🦊 Elysia is running at ${app.server?.hostname}:${app.server?.port}`
);
