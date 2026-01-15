import { Service } from "encore.dev/service";

export const greeting = new Service("greeting");

export const get = greeting.get({
    path: "/greeting/:name",
    expose: true,
}, async ({ name }: { name: string }) => {
    return { message: `Hello ${name}!` };
});
