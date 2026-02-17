# Local Debugging

Lambda Web Adapter lets you develop locally with familiar tools and debuggers — just run your web app and test it directly.

## Running Locally

Since the adapter only activates inside the Lambda execution environment, your app runs as a normal web server locally:

```bash
# Node.js
node index.js

# Python
uvicorn main:app --port 8080

# Java
./mvnw spring-boot:run
```

## Simulating Lambda with SAM CLI

To simulate the full Lambda runtime environment locally, use [AWS SAM CLI](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/install-sam-cli.html):

```bash
sam local start-api
```

> **Note:** `sam local` starts a Lambda Runtime Interface Emulator on port 8080. Your web application should avoid port 8080 if you plan to use `sam local`. Set `AWS_LWA_PORT` to a different port.

## Testing Individual Invocations

```bash
sam local invoke MyFunction --event event.json
```
