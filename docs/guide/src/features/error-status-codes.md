# Error Status Codes

You can configure specific HTTP status codes to cause Lambda invocations to fail, triggering Lambda's built-in error handling (retries, DLQ processing).

## Configuration

```
AWS_LWA_ERROR_STATUS_CODES=500,502-504,422
```

Supports individual codes and ranges, comma-separated.

## Behavior

When your web application returns any of the configured status codes, the Lambda invocation is marked as failed. This is useful for:

- Triggering automatic retries for transient errors
- Routing failed invocations to a Dead Letter Queue (DLQ)
- Integrating with Lambda Destinations for failure handling

This feature is disabled by default.
