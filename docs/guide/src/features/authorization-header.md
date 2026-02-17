# Authorization Header

When using Lambda Function URLs with [IAM auth type](https://docs.aws.amazon.com/lambda/latest/dg/urls-auth.html), the `Authorization` header is reserved for IAM authentication. If your backend app also needs an `Authorization` header, you can use a different header name and have the adapter rename it.

## Configuration

Set `AWS_LWA_AUTHORIZATION_SOURCE` to the header name your client will use:

```
AWS_LWA_AUTHORIZATION_SOURCE=X-Custom-Auth
```

The adapter will replace `X-Custom-Auth` with `Authorization` before forwarding the request to your app.

This feature is disabled by default.
