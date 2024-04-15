package com.amazonaws.demo.petstore.controller;

import com.amazonaws.demo.petstore.model.Pet;
import com.amazonaws.demo.petstore.model.PetData;
import io.javalin.http.Context;

import java.util.UUID;


public class PetsController {

    public static void createPet(Context ctx) {
        Pet newPet = ctx.bodyAsClass(Pet.class);

        if (newPet.getName() == null || newPet.getBreed() == null) {
            throw new IllegalArgumentException("Invalid name or breed");
        }

        Pet dbPet = newPet;
        dbPet.setId(UUID.randomUUID().toString());
        ctx.json(dbPet);
    }

    public static void listPets(Context ctx) { //@RequestParam("limit") Optional<Integer> limit, Principal principal) {
        Integer queryLimit = ctx.queryParamAsClass("limit", Integer.class).getOrDefault(10); // validate value

        Pet[] outputPets = new Pet[queryLimit];

        for (int i = 0; i < queryLimit; i++) {
            Pet newPet = new Pet();
            newPet.setId(UUID.randomUUID().toString());
            newPet.setName(PetData.getRandomName());
            newPet.setBreed(PetData.getRandomBreed());
            newPet.setDateOfBirth(PetData.getRandomDoB());
            outputPets[i] = newPet;
        }

        ctx.json(outputPets);
    }

    public static void getPet(Context ctx) {
        Pet newPet = new Pet();
        newPet.setId(UUID.randomUUID().toString());
        newPet.setBreed(PetData.getRandomBreed());
        newPet.setDateOfBirth(PetData.getRandomDoB());
        newPet.setName(PetData.getRandomName());
        ctx.json(newPet);
    }

}
