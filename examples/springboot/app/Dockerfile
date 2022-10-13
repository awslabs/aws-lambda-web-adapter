FROM public.ecr.aws/sam/build-java8.al2:latest as build-image
WORKDIR "/task"
COPY src src/
COPY pom.xml ./
RUN mvn -q clean package

FROM public.ecr.aws/docker/library/amazoncorretto:8u322-al2
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.5.0 /lambda-adapter /opt/extensions/lambda-adapter
EXPOSE 8080
WORKDIR /opt
COPY --from=build-image /task/target/petstore-0.0.1-SNAPSHOT.jar /opt
CMD ["java", "-jar", "petstore-0.0.1-SNAPSHOT.jar"]
