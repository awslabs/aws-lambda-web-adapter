FROM amd64/maven as build-image
WORKDIR "/task"
COPY src src/
COPY pom.xml ./
RUN mvn -q clean package

FROM public.ecr.aws/bitnami/java:1.8.292-prod
COPY --from=aws-lambda-adapter:latest /opt/bootstrap /opt/bootstrap
ENTRYPOINT ["/opt/bootstrap"]
EXPOSE 8080
WORKDIR /opt
COPY --from=build-image /task/target/petstore-0.0.1-SNAPSHOT.jar /opt
CMD ["java", "-jar", "petstore-0.0.1-SNAPSHOT.jar"]
