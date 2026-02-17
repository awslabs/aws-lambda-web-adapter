# Logging

Lambda Web Adapter supports [Lambda's Advanced Logging Controls](https://docs.aws.amazon.com/lambda/latest/dg/monitoring-logs.html#monitoring-cloudwatchlogs-advanced). Configure log level and format through the Lambda console, API, or CLI under **Monitoring and operations tools > Logging configuration**.

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `AWS_LAMBDA_LOG_LEVEL` | Log level: `DEBUG`, `INFO`, `WARN`, `ERROR`. Takes precedence over `RUST_LOG` | `INFO` |
| `AWS_LAMBDA_LOG_FORMAT` | Log format: `JSON` or `TEXT` | `TEXT` |

You can also set `RUST_LOG` as a fallback if `AWS_LAMBDA_LOG_LEVEL` is not configured.

## JSON Logging

When log format is set to `JSON`, log entries are formatted as JSON objects, making them easier to query with CloudWatch Logs Insights.
