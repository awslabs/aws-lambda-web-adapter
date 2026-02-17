# Request Interception

The `AWS_LWA_LAMBDA_RUNTIME_API_PROXY` environment variable redirects Lambda Runtime API requests to a custom proxy URL. The proxy can intercept requests and apply operations such as inspection, modification, tracing, or payload capturing.

## Configuration

```
AWS_LWA_LAMBDA_RUNTIME_API_PROXY=http://127.0.0.1:9002
```

## How It Works

- The proxy intercepts requests between the adapter and the Lambda Runtime API
- The event payload received by your web app is wrapped inside the GET response body
- This proxy does **not** affect the extension registration API
- It is meant only for interacting with data received and sent by the web application

## Use Cases

- Request/response tracing
- Payload capturing and logging
- Obfuscation of sensitive data
- Header modification
