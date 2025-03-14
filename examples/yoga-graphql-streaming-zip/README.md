# Yoga Graphql response streaming example

This example show how to use Lambda Web Adapter to run a yoga server application with response streaming via a [AWS Lambda](https://aws.amazon.com/lambda) Function URL.

### Build and Deploy

Run the following commands to build and deploy the application to lambda. 
```

```bash
sam build

sam deploy --guided
```
When the deployment completes, the Function URL will appear in the output list, which is the entrypoint for accessing

### Verify it works

When you open the Function URL in a browser:

1. Write and execute GraphQL queries in the left panel
2. See the results in the right panel
3. Explore the API documentation using the "Docs" button

Try this sample subscription query to test streaming:

```graphql
subscription {
  stream(addition: "testing")
}
```

You'll see each character stream in one by one with a small delay between them.

For command line testing, you can use curl:

```bash
curl --no-buffer -N \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{"query": "subscription { stream(addition: \"test\") }"}' \
  https://your-function-url.lambda-url.us-east-1.on.aws/graphql
```

### Thanks 

@sumcoding
