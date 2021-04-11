FROM amd64/maven as build-image
WORKDIR "/task"
COPY src src/
COPY pom.xml ./
RUN mvn -q clean package

FROM public.ecr.aws/lambda/java:8.al2
COPY --from=public.ecr.aws/awsguru/lambda-http-ric:latest /opt/bootstrap /opt/bootstrap
COPY --from=build-image /task/target/petstore-0.0.1-SNAPSHOT.jar /var/task/
ENTRYPOINT ["/opt/bootstrap"]
CMD ["java", "-jar", "petstore-0.0.1-SNAPSHOT.jar"]
