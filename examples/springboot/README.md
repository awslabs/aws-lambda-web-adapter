# Spring Boot 2 example

A basic pet store application written with the Spring Boot 2 framework. You can build and test it locally as a typical Spring Boot 2 application. 

Using AWS Lambda Adapter, you can package this web application into Docker image, push to ECR, and deploy to Lambda, ECS/EKS, or EC2.

The application can be deployed in an AWS account using the [Serverless Application Model](https://github.com/awslabs/serverless-application-model). The `template.yaml` file in the root folder contains the application definition.

The top level folder is a typical AWS SAM project. The `app` directory is a Spring Boot application with a multi-stage [Dockerfile](app/Dockerfile).

```dockerfile
FROM public.ecr.aws/sam/build-java8.al2:latest as build-image
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
```

Line 7 and 8 copy lambda adapter binary and set it as ENTRYPOINT. This is the only change to run the Spring Boot application on Lambda.

```dockerfile
COPY --from=aws-lambda-adapter:latest /opt/bootstrap /opt/bootstrap
ENTRYPOINT ["/opt/bootstrap"]
```

## Pre-requisites

The following tools should be installed and configured.

* [AWS CLI](https://aws.amazon.com/cli/)
* [SAM CLI](https://github.com/awslabs/aws-sam-cli)
* [Maven](https://maven.apache.org/)
* [Docker](https://www.docker.com/products/docker-desktop)

Container image `aws-lambda-adapter:latest` should already exist. You can follow [README](../../README.md#how-to-build-it?) to build Lambda Adapter.

## Deploy to Lambda
Navigate to the sample's folder and use the SAM CLI to build a container image
```shell
$ aws ecr-public get-login-password --region us-east-1 | docker login --username AWS --password-stdin public.ecr.aws
$ sam build
```

This command compiles the application and prepares a deployment package in the `.aws-sam` sub-directory.

To deploy the application in your AWS account, you can use the SAM CLI's guided deployment process and follow the instructions on the screen

```shell
$ sam deploy --guided
```
Please take note of the container image name. 
Once the deployment is completed, the SAM CLI will print out the stack's outputs, including the new application URL. You can use `curl` or a web browser to make a call to the URL

```shell
...
---------------------------------------------------------------------------------------------------------
OutputKey-Description                        OutputValue
---------------------------------------------------------------------------------------------------------
PetStoreApi - URL for application            https://xxxxxxxxxx.execute-api.us-west-2.amazonaws.com/pets
---------------------------------------------------------------------------------------------------------
...

$ curl https://xxxxxxxxxx.execute-api.us-west-2.amazonaws.com/pets
```

## Run the docker locally

We can run the same docker image locally, so that we know it can be deployed to ECS Fargate and EKS EC2 without code changes. 

```shell
$ docker run -d -p 8080:8080 {ECR Image}

```

Use curl to verify the docker container works. 

```shell
$ curl localhost:8080/pets 
```
