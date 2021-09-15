# nginx example

A basic nginx web server runs inside AWS Lambda. 

Using AWS Lambda Adapter, you can package this web server into Docker image, push to ECR, and deploy to Lambda, ECS/EKS, or EC2.

The application can be deployed in an AWS account using the [Serverless Application Model](https://github.com/awslabs/serverless-application-model). The `template.yaml` file in the root folder contains the application definition.

The top level folder is a typical AWS SAM project. The `app` directory is the nginx configuration with a [Dockerfile](app/Dockerfile).

```dockerfile
FROM public.ecr.aws/runecast/nginx:1.21.1-alpine
COPY --from=aws-lambda-adapter:latest /opt/bootstrap /opt/bootstrap
ENTRYPOINT ["/opt/bootstrap"]
WORKDIR "/tmp"
ADD config/ /etc/nginx/
ADD images/ /usr/share/nginx/html/images
CMD ["nginx", "-g", "daemon off;"]
```

Line 2 and 3 copy lambda adapter binary and set it as ENTRYPOINT. This is the main change to run the nginx server on Lambda. 

## Pre-requisites

The following tools should be installed and configured.

* [AWS CLI](https://aws.amazon.com/cli/)
* [SAM CLI](https://github.com/awslabs/aws-sam-cli)
* [Docker](https://www.docker.com/products/docker-desktop)

Container image `aws-lambda-adapter:latest` should already exist. You could follow [README](../../README.md#how-to-build-it?) to build Lambda Adapter.

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

Lambda Adapter also automatic encode/decode binary data for you. Open the output link in your browser, add "images/space.jpeg" to the url, you will see a picture of the space. 

https://xxxxxxxxxx.execute-api.us-west-2.amazonaws.com/images/space.jpeg

![space](app/images/space.jpeg)

## Run the docker locally

We can run the same docker image locally, so that we know it can be deployed to ECS Fargate and EKS EC2 without code changes.

```shell
$ docker run -d -p 8080:8080 {ECR Image}

```

Use curl to verify the docker container works.

```shell
$ curl localhost:8080/ 
```
