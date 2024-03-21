import { OpenAPIV3 } from "openapi-types/dist/index";

export const tags: OpenAPIV3.TagObject[] = [
    { name: 'Auth', description: 'Authentication' },
    { name: 'User', description: 'User' },
    { name: 'Admin', description: 'Admin' },
];

export const documentation: Omit<Partial<OpenAPIV3.Document>, 'x-express-openapi-additional-middleware' | 'x-express-openapi-validation-strict'> = {
    tags,
};
