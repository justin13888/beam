import { JWT_ACCESS_TOKEN_DURATION, JWT_ISSUER, JWT_REFRESH_TOKEN_DURATION } from "@/lib/constants";
import { privateKey, publicKey } from "@/lib/jwt";
import bearer from "@elysiajs/bearer";
import { jwt } from "@elysiajs/jwt";
import { Elysia, Static, t } from "elysia";

export const Scope = t.Union([
    // Affect own profile
    t.Literal('profile:own'),
    // Read media
    t.Literal('media:read'),
    // Read account
    t.Literal('account:read'),
    // Modify account
    t.Literal('account:modify'),
    // Delete account
    t.Literal('account:delete'),
    // Admin access
    t.Literal('sudo'),
]);
type ScopeType = Static<typeof Scope>;
export const Scopes = t.Record(Scope, t.Boolean());
export type ScopesType = Static<typeof Scopes>;
export const defaultScopes: ScopesType = {
    'profile:own': false,
    'media:read': false,
    'account:read': false,
    'account:modify': false,
    'account:delete': false,
    'sudo': false,
};

export const jwtAccessSetup = new Elysia().use(jwt({
    name: 'jwtAccess',
    alg: 'HS512',
    secret: 'secret', // TODO: Change back to EdDSA when supported by @elysiajs/jwt
    // alg: JWT_ALGORITHM,
    // privateKey,
    // publicKey,
    schema: t.Object({
        id: t.String(),
        scopes: Scopes,
    }),
    iss: JWT_ISSUER,
    exp: `${JWT_ACCESS_TOKEN_DURATION}s`,
    type: 'access',
}));

export const jwtRefreshSetup = new Elysia().use(jwt({
    name: 'jwtRefresh',
    alg: 'HS512',
    secret: 'secret', // TODO: Change back to EdDSA when supported by @elysiajs/jwt
    // alg: JWT_ALGORITHM,
    // privateKey,
    // publicKey,
    schema: t.Object({
        id: t.String(),
    }),
    iss: JWT_ISSUER,
    exp: `${JWT_REFRESH_TOKEN_DURATION}s`,
    type: 'refresh',
}));

export const isAuthorized = (requiredScopes: ScopesType) => 
    new Elysia()
        .use(bearer())
        .use(jwtAccessSetup)
        .derive(async ({ jwtAccess, bearer, set }) => {
            const user = await jwtAccess.verify(bearer);
            if (!user) {
                set.status = 401;
                throw new Error('Unauthorized');
            }

            for (const scope in requiredScopes) {
                if (!user.scopes[scope as keyof ScopesType]) {
                    throw new Error('Unauthorized');
                }
            }

            return {
                user,
            };
        })
