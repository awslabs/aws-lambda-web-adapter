package com.amazonaws.demo.petstore;

import com.amazonaws.demo.petstore.controller.PetsController;
import io.javalin.Javalin;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static io.javalin.apibuilder.ApiBuilder.*;

public class Application {
    private static final Logger log = LoggerFactory.getLogger(Application.class);

    public static void main(String[] args) {

        var app = Javalin.create(config -> {
            config.requestLogger.http((ctx, ms) -> {
                log.debug("Request.path: [{}] {}, Result.statusCode: {}, Body:\n{}\n", ctx.method(), ctx.fullUrl(), ctx.statusCode(), ctx.result());
            });
            config.router.apiBuilder(() -> {
                get("/", ctx -> ctx.result("Hello, world!"));
                get("/healthz", ctx -> ctx.result("healthy"));
                path("/pets", () -> {
                    get(PetsController::listPets);
                    post(PetsController::createPet);
                    path("/{id}", () -> {
                        get(PetsController::getPet);
                    });
                });
            });
        });

        app.start("127.0.0.1", 8081);
    }
}