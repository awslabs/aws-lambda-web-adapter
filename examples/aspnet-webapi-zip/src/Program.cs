using System.Text.Json;

using Amazon.Lambda.APIGatewayEvents;

using Microsoft.AspNetCore.Mvc;

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddEndpointsApiExplorer();

var app = builder.Build();

var summaries = new[]
{
    "Freezing", "Bracing", "Chilly", "Cool", "Mild", "Warm", "Balmy", "Hot", "Sweltering", "Scorching"
};

app.MapGet("/weatherforecast", () =>
{
    var forecast =  Enumerable.Range(1, 5).Select(index =>
        new WeatherForecast
        (
            DateTime.Now.AddDays(index),
            Random.Shared.Next(-20, 55),
            summaries[Random.Shared.Next(summaries.Length)]
        ))
        .ToArray();
    return forecast;
})
.WithName("GetWeatherForecast");

app.MapGet(
        "/context",
        ([FromHeader(Name = "x-amzn-request-context")] string requestContext, [FromHeader(Name = "x-amzn-lambda-context")] string lambdaContext) =>
        {
            var jsonOptions = new JsonSerializerOptions()
            {
                PropertyNameCaseInsensitive = true,
            };

            LambdaContext parsedLambdaContext = null;
            APIGatewayHttpApiV2ProxyRequest.ProxyRequestContext parsedReqContext = null;

            if (!string.IsNullOrEmpty(requestContext))
            {
                parsedReqContext = JsonSerializer.Deserialize<APIGatewayHttpApiV2ProxyRequest.ProxyRequestContext>(
                    requestContext,
                    jsonOptions);
            }

            if (!string.IsNullOrEmpty(lambdaContext))
            {
                parsedLambdaContext = JsonSerializer.Deserialize<LambdaContext>(
                    lambdaContext,
                    jsonOptions);
            }

            return new
            {
                lambdaContext = parsedLambdaContext,
                requestContext = parsedReqContext
            };
        })
    .WithName("Context");

app.Run();