import { Elysia, t } from "elysia";

export const tokenModel = new Elysia().model({
    token: t.String(),
});

export const accessTokenModel = new Elysia().model({
    accessToken: t.Object({
        access_token: t.String(),
    }),
});

export const refreshTokenModel = new Elysia().model({
    refreshToken: t.Object({
        refresh_token: t.String(),
    }),
});
