#!/bin/sh

java -cp "./:lib/*" "-XX:TieredStopAtLevel=1" "com.amazonaws.demo.petstore.Application"