# NestJS Example with Serverless Framework

Basic example of using the `aws-lambda-web-adapter` package with the NestJS framework, and the Serverless Framework version 3.

## Overview

The article ["Container Image Support for AWS Lambda"](https://www.serverless.com/blog/container-support-for-lambda) on the Serverless Framework blog introduces the ability to use container images with AWS Lambda, allowing developers to have full control over the execution environment and access to custom runtimes and libraries.

## Getting Started

Follow these steps to get the example up and running:

1. Install the project dependencies using pnpm / npm:

   ```bash
   pnpm install
   ```

2. Deploy the application using the Serverless Framework:

   ```bash
   serverless deploy
   ```

   This command will deploy the application to your AWS account using the Serverless Framework and create the necessary AWS Lambda function.

3. Test the endpoint:

   Once the deployment is complete, the Serverless Framework will provide you with the endpoint URL. You can test the endpoint by sending an HTTP request to that URL. You should receive a "Hello, World!" response.


To tear down the app, use:

   ```bash
   serverless remove
   ```

   This command will performs the following actions:
   - Deletes the deployed AWS Lambda functions and associated resources.
   - Cleans up any event sources or triggers associated with the functions.
   - Deletes any additional resources provisioned by the Serverless Framework, such as AWS CloudFormation stacks or other infrastructure components.

## Author
- [Lafif Astahdziq](https://lafif.me)