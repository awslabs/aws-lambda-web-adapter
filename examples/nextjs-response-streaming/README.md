# Nextjs response streaming example

This example show how to use Lambda Web Adapter to run a nextjs application with response streaming via a [AWS Lambda](https://aws.amazon.com/lambda) Function URL.

### Build and Deploy

Run the following commands to build and deploy the application to lambda. 

```bash
sam build

sam deploy --guided
```
When the deployment completes, the Function URL will appear in the output list, which is the entrypoint for accessing

### Verify it works

When you open the Function URL in a browser:

- Primary product information will be loaded first at part of the initial response

- Secondary, more personalized details (that might be slower) like recommended products and customer reviews are progressively streamed in.
