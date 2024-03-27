import { Elysia, t } from "elysia";

export const newUserModel = new Elysia().model({
    newUser: t.Object({
        email: t.String({ format: "email" }),
        username: t.String({ minLength: 1 }),
        password: t.String({ minLength: 1 }),
    }),
});

export const userCredentialsModel = new Elysia().model({
    userCredentials: t.Object({
        username: t.String(),
        password: t.String(),
    }),
});
