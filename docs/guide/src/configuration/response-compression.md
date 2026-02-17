# Response Compression

Lambda Web Adapter supports gzip and Brotli compression for response bodies in buffered mode.

## Enabling Compression

```
AWS_LWA_ENABLE_COMPRESSION=true
```

## Behavior

When enabled:

- Responses are compressed using gzip or Brotli based on the client's `Accept-Encoding` header
- Responses with `Content-Type` starting with `image` are **not** compressed
- Responses smaller than 32 bytes are **not** compressed

## Limitations

Compression is **not supported** with response streaming (`AWS_LWA_INVOKE_MODE=response_stream`). If both are enabled, compression is automatically disabled and a warning is logged.
