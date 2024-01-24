# Agents for Amazon Bedrock with FastAPI example

This project demonstrates the integration of "Agents for Amazon Bedrock" with a FastAPI application on AWS Lambda. It showcases how to effectively manage and  process Agents for Amazon Bedrock within an serverless FastAPI application environment.

The application can be deployed in an AWS account using the [Serverless Application Model](https://github.com/awslabs/serverless-application-model). The `template.yaml` file in the root folder contains the application definition.

The top level folder is a typical AWS SAM project. The `app` directory is an FastAPI application with a [Dockerfile](app/Dockerfile).

```dockerfile
FROM public.ecr.aws/docker/library/python:3.12.0-slim
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.8.1 /lambda-adapter /opt/extensions/lambda-adapter

WORKDIR /app
ADD . .
RUN pip install -r requirements.txt

CMD ["python", "main.py"]
```

Line 2 copies lambda adapter binary into /opt/extenions. This is the only change to run the FastAPI application on Lambda.

```dockerfile
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.8.1 /lambda-adapter /opt/extensions/lambda-adapter
```

## Pre-requisites

The following tools should be installed and configured.
* [AWS CLI](https://aws.amazon.com/cli/)
* [SAM CLI](https://github.com/awslabs/aws-sam-cli)
* [Python](https://www.python.org/)
* [Docker](https://www.docker.com/products/docker-desktop)

## Deploy to Lambda
Navigate to the sample's folder and use the SAM CLI to build a container image
```shell
$ sam build
```

This command compiles the application and prepares a deployment package in the `.aws-sam` sub-directory.

To deploy the application in your AWS account, you can use the SAM CLI's guided deployment process and follow the instructions on the screen

```shell
$ sam deploy --guided
```

Create a bedrock agent. (see [reference](https://docs.aws.amazon.com/bedrock/latest/userguide/agents-create.html
))

## Test

Test your agent. (see [reference](https://docs.aws.amazon.com/bedrock/latest/userguide/agents-test.html))

## Run the docker locally

We can run the same docker image locally, so that we know it can be deployed to ECS Fargate and EKS EC2 without code changes.

```shell
$ docker run -d -p 8000:8000 {ECR Image}

```

Use curl to verify the docker container works.

```shell
$ curl localhost:8000/ 
```
