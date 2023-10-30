using System.Text.Json.Serialization;

record LambdaEnvironment
{
    /// <inheritdoc />
    [JsonPropertyName("function_name")]
    public string FunctionName { get; set; }

    /// <inheritdoc />
    [JsonPropertyName("memory")]
    public int MemoryLimitInMB { get; set; }
    
    /// <inheritdoc />
    [JsonPropertyName("version")]
    public string FunctionVersion { get; set; }

    /// <inheritdoc />
    [JsonPropertyName("log_group")]
    public string LogGroupName { get; set; }

    /// <inheritdoc />
    [JsonPropertyName("log_stream")]
    public string LogStreamName { get; set; }
}