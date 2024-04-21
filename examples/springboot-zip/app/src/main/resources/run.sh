#!/bin/sh

exec java -cp "./:lib/*" "com.amazonaws.demo.petstore.Application" "--server.port=${PORT}"