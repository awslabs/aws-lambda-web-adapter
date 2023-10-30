using System.Text.Json.Serialization;

record LambdaContext
{
    /// <inheritdoc />
    [JsonPropertyName("request_id")]
    public string AwsRequestId { get; set; }
    
    /// <inheritdoc />
    [JsonPropertyName("deadline")]
    public long Deadline { get; set; }

    /// <inheritdoc />
    [JsonPropertyName("invoked_function_arn")]
    public string InvokedFunctionArn { get; set; }

    /// <inheritdoc />
    [JsonPropertyName("xray_trace_id")]
    public string XRayTraceId { get; set; }
    
    [JsonPropertyName("env_config")]
    public LambdaEnvironment Environment { get; set; }
}