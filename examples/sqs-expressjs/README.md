# SQS Express.js example

This project demonstrates the integration of Amazon Simple Queue Service (SQS) with an Express.js application. It showcases how to effectively manage and process messages from an SQS queue within an Express.js application environment.

In this Express.js application integrated with Amazon SQS, there is no explicit code required to poll the SQS queue. The AWS Lambda handles the polling of the SQS queue and Lambda Web Adapter forwards the event payload to the Express.js application. This simplifies the application code and allows developers to focus on processing the event payload rather than managing the queue polling logic.

The application can be deployed in an AWS account using the [Serverless Application Model](https://github.com/awslabs/serverless-application-model). The `template.yaml` file in the root folder contains the application definition.

The top level folder is a typical AWS SAM project. The `app` directory is an express.js application with a [Dockerfile](app/Dockerfile).

```dockerfile
FROM public.ecr.aws/docker/library/node:20-slim
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.8.0 /lambda-adapter /opt/extensions/lambda-adapter
ENV PORT=8000 AWS_LWA_READINESS_CHECK_PROTOCOL=tcp
WORKDIR "/var/task"
ADD src/package.json /var/task/package.json
ADD src/package-lock.json /var/task/package-lock.json
RUN npm install --omit=dev
ADD src/ /var/task
CMD ["node", "index.js"]
```

Line 2 copies lambda adapter binary into /opt/extenions. This is the only change to run the express.js application on Lambda.

```dockerfile
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.8.0 /lambda-adapter /opt/extensions/lambda-adapter
```

## Pre-requisites

The following tools should be installed and configured.

* [AWS CLI](https://aws.amazon.com/cli/)
* [SAM CLI](https://github.com/awslabs/aws-sam-cli)
* [Node](https://nodejs.org/en/)
* [Docker](https://www.docker.com/products/docker-desktop)

## Build

Navigate to the sample's folder and use the SAM CLI to build a container image

```shell
sam build
```
## Deploy

This command compiles the application and prepares a deployment package in the `.aws-sam` sub-directory.

To deploy the application in your AWS account, you can use the SAM CLI's guided deployment process and follow the instructions on the screen

```shell
sam deploy --guided
```

Please take note of the container image name.
Once the deployment is completed, the SAM CLI will print out the stack's outputs, including the new sqs queue URL.

```shell
...
---------------------------------------------------------------------------------------------------------------------------------------
Outputs                                                                                                                               
---------------------------------------------------------------------------------------------------------------------------------------
Key                 SqsQueueUrl                                                                                                       
Description         SQS URL the express Lambda Function will receive messages from                                                    
Value               https://sqs.us-west-2.amazonaws.com/xxxxxxxx/xxxxxxxx                                                    
---------------------------------------------------------------------------------------------------------------------------------------

```

## Test

Use the following command to send a message to the sqs queue.

```shell
aws sqs send-message --queue-url <replace with your sqs queue url> --message-body "Hello from CLI"
```

Run the following command to see the Lambda function's CloudWatch logs.

```shell
sam logs --tail --stack-name <replace with your stack name>
```

## Local Test

You can also use SAM CLI for local testing.

```shell
sam local invoke SqsExpressFunction -e events/sqs.json
```

Here is a sample output from this command.

```shell
Invoking Container created from sqsexpressfunction:v1                                                                            
Building image.................
Using local image: sqsexpressfunction:rapid-x86_64.                                                                              
                                                                                                                                 
START RequestId: ceaaf9bb-8d8c-42a5-828c-a5d4c8a506f1 Version: $LATEST
Example app listening at http://localhost:8000
Received event: {"Records":[{"messageId":"19dd0b57-b21e-4ac1-bd88-01bbb068cb78","receiptHandle":"MessageReceiptHandle","body":"Hello from SQS!","attributes":{"ApproximateReceiveCount":"1","SentTimestamp":"1523232000000","SenderId":"123456789012","ApproximateFirstReceiveTimestamp":"1523232000001"},"messageAttributes":{},"md5OfBody":"7b270e59b47ff90a553787216d55d91d","eventSource":"aws:sqs","eventSourceARN":"arn:aws:sqs:us-east-1:123456789012:MyQueue","awsRegion":"us-east-1"}]}
processing message: 19dd0b57-b21e-4ac1-bd88-01bbb068cb78 with body: Hello from SQS!
END RequestId: ceaaf9bb-8d8c-42a5-828c-a5d4c8a506f1
REPORT RequestId: ceaaf9bb-8d8c-42a5-828c-a5d4c8a506f1  Init Duration: 0.10 ms  Duration: 117.12 ms     Billed Duration: 118 ms Memory Size: 1024 MB     Max Memory Used: 1024 MB
"success"
```
