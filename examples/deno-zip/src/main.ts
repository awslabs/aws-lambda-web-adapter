import { Application, Router } from "https://deno.land/x/oak/mod.ts";

const app = new Application();
const router = new Router();
const port = +(Deno.env.get("PORT") || "8080")

router.get("/", (context) => {
    context.response.body = {
        success: true,
        msg: "Hello World",
    };
});

app.use(router.routes());

await app.listen({ port });
