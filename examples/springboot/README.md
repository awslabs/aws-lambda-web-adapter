# Spring Boot 2 example

A basic pet store application written with the Spring Boot 2 framework. The application can be build and tested as a typical Spring Boot 2 application. 

Using AWS Lambda Adapter, we can package this web application into Docker image, push to ECR, and deploy to Lambda, ECS Fargate, or EKS EC2.

The application can be deployed in an AWS account using the [Serverless Application Model](https://github.com/awslabs/serverless-application-model). The `template.yaml` file in the root folder contains the application definition.

## Pre-requisites
* [AWS CLI](https://aws.amazon.com/cli/)
* [SAM CLI](https://github.com/awslabs/aws-sam-cli)
* [Gradle](https://gradle.org/) or [Maven](https://maven.apache.org/)
* [jq](https://stedolan.github.io/jq/)

## Create ECR repository

We need to create an ECR repository for storing the docker image. We can do it with AWS CLI. 

```shell
ECR_REPO=`aws ecr create-repository --repository-name springbootapp --region ap-southeast-1 | jq -r .repository.repositoryUri`
echo $ECR_REPO
```

## Deploy to Lambda
Then, navigate to the sample's folder and use the SAM CLI to build a deployable package
```
$ sam build
```

This command compiles the application and prepares a deployment package in the `.aws-sam` sub-directory.

To deploy the application in your AWS account, you can use the SAM CLI's guided deployment process and follow the instructions on the screen

```
$ sam deploy --guided
```
Please take note of the container image name. 
Once the deployment is completed, the SAM CLI will print out the stack's outputs, including the new application URL. You can use `curl` or a web browser to make a call to the URL

```
...
---------------------------------------------------------------------------------------------------------
OutputKey-Description                        OutputValue
---------------------------------------------------------------------------------------------------------
PetStoreApi - URL for application            https://xxxxxxxxxx.execute-api.us-west-2.amazonaws.com/pets
---------------------------------------------------------------------------------------------------------

$ curl https://xxxxxxxxxx.execute-api.us-west-2.amazonaws.com/pets
```

## Run the docker locally

We can run the same docker image locally, so that we know it can be deployed to ECS Fargate and EKS EC2 without code changes. 

```shell
docker run -d -p 8080:8080 {ECR Image}

```

Use curl to verify the docker container works. 

```shell
curl localhost:8080/pets 
```
