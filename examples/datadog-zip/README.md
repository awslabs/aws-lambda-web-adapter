# Datadog support for Lambda Web Adapter

This folder contains examples using Lambda Web Adapter and Datadog instrumentation in nodejs (expressjs) using a zipfile and layers instead of a container.

The examples use aws cdk to deploy and curl to test, so make sure they are installed.

# How to use

Instructions are for expressjs

## Install dependencies

Install aws cdk dependencies

```sh
cd expressjs/cdk
npm i
cd -
```

## Deploy and Run

Deploy with

```sh
cd expressjs/cdk
cdk deploy
cd -
```

After confirming the deployment, a log will show a public Lambda URL to invoke the endpoint with

```sh
Outputs:
lwa-stack.LambdaFunctionUrl = https://xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx.lambda-url.us-east-1.on.aws/
```

and the function can be invoked with (note the call_lwa at the end of the URL)

```sh
curl https://xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx.lambda-url.us-east-1.on.aws/call_lwa
```

NB
this deployment will create a publicly accessible URL link with no security restriction and usage limits! Make sure to run

```sh
cd expressjs/cdk
cdk destroy
cd -
```

after the example test is done
