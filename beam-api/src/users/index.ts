import { Elysia, t } from "elysia";
import { bearer } from '@elysiajs/bearer'

// TODO
const users = new Elysia({ prefix: '/users' })
    .post('/sign-in', ({body}) => body, {
        body: t.Object(
            {
                username: t.String(),
                password: t.String()
            },
            {
                description: 'Expected username and password',
            }
        ),
        detail: {
            tags: ['Auth'],
            summary: 'Sign in',
            description: 'Sign in to get refresh token'
        }
    })
    // .use(bearer())
    // .get('/sign', ({ bearer }) => bearer, {
    //     beforeHandle({ bearer, set }) {
    //         if (!bearer) {
    //             set.status = 400
    //             set.headers[
    //                 'WWW-Authenticate'
    //             ] = `Bearer realm='sign', error="invalid_request"`

    //             return 'Unauthorized'
    //         }
    //     }
    // })

export default users;
