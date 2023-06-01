package com.amazonaws.demo.stream.controller;

import org.springframework.core.io.InputStreamResource;
import org.springframework.core.io.Resource;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.servlet.config.annotation.EnableWebMvc;

import java.io.*;

@RestController
@EnableWebMvc
public class StreamFilesController {


    @GetMapping("/stream")
    public ResponseEntity<Resource> streamFile() throws IOException {
        // Load the file from the filesystem or any other source
        File file = new File("room.mp4");

        // Check if the file exists
        if (!file.exists()) {
            // Return an error response if the file does not exist
            return ResponseEntity.notFound().build();
        }

        // Create an InputStreamResource from the file
        InputStreamResource resource = new InputStreamResource(new FileInputStream(file));

        // Set the response headers
        HttpHeaders headers = new HttpHeaders();
        headers.add(HttpHeaders.CONTENT_DISPOSITION, "attachment; filename=" + file.getName());

        // Stream the file as the response
        return ResponseEntity.ok()
                .headers(headers)
                .contentLength(file.length())
                .contentType(MediaType.APPLICATION_OCTET_STREAM)
                .body(resource);
    }

    @GetMapping("/stream-dummy")
    public ResponseEntity<InputStreamResource> streamDummyFile(@RequestParam("size") long fileSize) {
        // Create a byte array with the specified size
        byte[] dummyData = new byte[(int) fileSize*1024*1024];
        System.out.println("Size "+fileSize+" array size :"+dummyData.length);

        // Create an InputStream from the byte array
        InputStream inputStream = new ByteArrayInputStream(dummyData);

        // Set the response headers
        HttpHeaders headers = new HttpHeaders();
        headers.add(HttpHeaders.CONTENT_DISPOSITION, "attachment; filename=dummy_file.bin");

        // Stream the dummy file as the response
        return ResponseEntity.ok()
                .headers(headers)
                .contentLength(fileSize*1024*1024)
                .contentType(MediaType.APPLICATION_OCTET_STREAM)
                .body(new InputStreamResource(inputStream));
    }

}
