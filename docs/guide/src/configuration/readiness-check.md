# Readiness Check

When a new Lambda execution environment starts, the adapter boots as a Lambda Extension, followed by your web application. The adapter needs to know when your app is ready to receive traffic.

## How It Works

1. The adapter sends HTTP GET requests to your app at `http://127.0.0.1:8080/`
2. It retries every 10 milliseconds
3. Once it receives an HTTP response with status code >= 100 and < 500, the app is considered ready
4. The adapter then starts the Lambda runtime client and begins forwarding invocations

## Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `AWS_LWA_READINESS_CHECK_PORT` | Port to check | Same as `AWS_LWA_PORT` |
| `AWS_LWA_READINESS_CHECK_PATH` | Path to check | `/` |
| `AWS_LWA_READINESS_CHECK_PROTOCOL` | `http` or `tcp` | `http` |
| `AWS_LWA_READINESS_CHECK_HEALTHY_STATUS` | Status codes considered healthy | `100-499` |

## TCP Readiness Check

If your app doesn't have an HTTP health endpoint ready at startup, you can use TCP-based readiness checking:

```
AWS_LWA_READINESS_CHECK_PROTOCOL=tcp
```

This checks only that the port is accepting TCP connections.

## Custom Health Status Codes

You can customize which HTTP status codes are considered healthy:

```
# Only 2xx and 3xx are healthy
AWS_LWA_READINESS_CHECK_HEALTHY_STATUS=200-399

# Specific codes
AWS_LWA_READINESS_CHECK_HEALTHY_STATUS=200,201,204,301-399
```
