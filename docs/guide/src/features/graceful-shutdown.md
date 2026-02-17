# Graceful Shutdown

When Lambda is about to shut down an execution environment, it sends a `SIGTERM` signal to the runtime and a `SHUTDOWN` event to each registered extension. Your application can catch `SIGTERM` to perform cleanup tasks.

## Example: Express.js

```javascript
const server = app.listen(port, () => {
    console.log(`Listening on port ${port}`);
});

process.on('SIGTERM', () => {
    console.info('SIGTERM received, shutting down gracefully');
    server.close(() => {
        console.info('Server closed');
        process.exit(0);
    });
});
```

## Use Cases

- Close database connections
- Flush logs or metrics
- Complete in-progress requests
- Release external resources

For more details, see the [graceful shutdown with AWS Lambda](https://github.com/aws-samples/graceful-shutdown-with-aws-lambda) repository.
