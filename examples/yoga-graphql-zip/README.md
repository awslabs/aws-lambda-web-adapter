# Yoga Graphql API example

This example show how to use Lambda Web Adapter to run a yoga server application with via [AWS Lambda](https://aws.amazon.com/lambda) and API Gateway.

### Build and Deploy

Run the following commands to build and deploy the application to lambda. 
```

```bash
sam build

sam deploy --guided
```
When the deployment completes, the API URL will appear in the output list, which is the entrypoint for accessing

### Verify it works

When you open the API URL in a browser:

1. Write and execute GraphQL queries in the left panel
2. See the results in the right panel
3. Explore the API documentation using the "Docs" button

Try this sample subscription query to test streaming:

```graphql
query {
  feed {
    id
    description
    url
  }
}
```

You'll see each character stream in one by one with a small delay between them.

For command line testing, you can use curl:

```bash
curl -N \
  -H "Content-Type: application/json" \
  -d '{"query": "query { feed {id url description} }"}' \
  https://<api-id>.execute-api.us-east-1.amazonaws.com/graphql
```

### Thanks 

@sumcoding
