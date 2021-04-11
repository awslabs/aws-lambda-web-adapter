package com.amazonaws.demo.petstore;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.boot.web.servlet.server.Session;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;

import javax.servlet.http.Cookie;
import javax.servlet.http.HttpServletResponse;

@SpringBootApplication
@RestController
public class PetstoreApplication {

    @GetMapping("/")
    public String index() {
        return "Hi there!";
    }

    @GetMapping(path = "/hello", produces = "application/json")
    public String hello() {
        return "Hello, world!";
    }

    @GetMapping("/cookie")
    public String seCcookie(HttpServletResponse response) {
        Cookie cookie = new Cookie("username", "Harold");
        response.addCookie(cookie);
        return "cookie is set.";
    }

    @GetMapping("/healthz")
    public String healthCheck() {
        return "healthy";
    }

    public static void main(String[] args) {
        SpringApplication.run(PetstoreApplication.class, args);
    }
}
