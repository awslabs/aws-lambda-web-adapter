# Express.js example

A basic express.js application example. You can build and test it locally as a typical Express.js application.

Using AWS Lambda Adapter, you can package this web application into Docker image, push to ECR, and deploy to Lambda, ECS/EKS, or EC2.

The application can be deployed in an AWS account using the [Serverless Application Model](https://github.com/awslabs/serverless-application-model). The `template.yaml` file in the root folder contains the application definition.

The top level folder is a typical AWS SAM project. The `app` directory is an express.js application with a [Dockerfile](app/Dockerfile). 

```dockerfile
FROM public.ecr.aws/docker/library/node:16.13.2-stretch-slim
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.9.1 /lambda-adapter /opt/extensions/lambda-adapter
EXPOSE 8080
WORKDIR "/var/task"
ADD src/package.json /var/task/package.json
ADD src/package-lock.json /var/task/package-lock.json
RUN npm install --omit=dev
ADD src/ /var/task
CMD ["node", "index.js"]
```

Line 2 copies lambda adapter binary into /opt/extenions. This is the only change to run the express.js application on Lambda.

```dockerfile
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.9.1 /lambda-adapter /opt/extensions/lambda-adapter
```

## Pre-requisites

The following tools should be installed and configured. 
* [AWS CLI](https://aws.amazon.com/cli/)
* [SAM CLI](https://github.com/awslabs/aws-sam-cli)
* [Node](https://nodejs.org/en/)
* [Docker](https://www.docker.com/products/docker-desktop)


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
PetStoreApi - URL for application            https://xxxxxxxxxx.execute-api.us-west-2.amazonaws.com/
---------------------------------------------------------------------------------------------------------
...

$ curl https://xxxxxxxxxx.execute-api.us-west-2.amazonaws.com/
```

## Run the docker locally

We can run the same docker image locally, so that we know it can be deployed to ECS Fargate and EKS EC2 without code changes.

```shell
$ docker run -d -p 8080:8080 {ECR Image}

```

Use curl to verify the docker container works.

```shell
$ curl localhost:8080/ 
```
