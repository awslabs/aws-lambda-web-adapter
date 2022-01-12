# expressjs-zip

This example shows how to use Lambda Adapter to run an express.js application on managed node.js runtime. 

### How does it work?

We package Lambda Adapter binary and a wrapper script into a Lambda Layer. And we configure environment variable 'AWS_LAMBDA_EXEC_WRAPPER' pointing to the wrapper script.

When Lambda execution environment starts, Lambda will execute the wrapper script and starts the adapter. 

To get more information of Wrapper script, please read Lambda documentation [here](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-modify.html#runtime-wrapper). 

### Build and Deploy

Run the following commands to build and deploy the application to lambda. 

```bash
sam build
sam deploy --guided
```
When the deployment completes, take note of HelloWorldApi's Value. It is the API Gateway endpoint URL. 

### Verify it works

Open HelloWorldApi's URL in a browser, you should see "Hi there!" on the page. 


