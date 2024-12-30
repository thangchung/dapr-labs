using Dapr;
using Dapr.Client;
using System.Diagnostics;
using System.Text.Json;

var builder = WebApplication.CreateBuilder(args);

builder.AddServiceDefaults();

// Add services to the container.
// Learn more about configuring OpenAPI at https://aka.ms/aspnet/openapi
builder.Services.AddOpenApi();

builder.Services.AddDaprClient();
builder.Services.AddSingleton(new JsonSerializerOptions()
{
    PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
    PropertyNameCaseInsensitive = true,
});

var app = builder.Build();

app.MapDefaultEndpoints();

// Configure the HTTP request pipeline.
if (app.Environment.IsDevelopment())
{
    app.MapOpenApi();
}

app.UseRouting();

app.UseCloudEvents();

app.MapSubscribeHandler();

var summaries = new[]
{
    "Freezing", "Bracing", "Chilly", "Cool", "Mild", "Warm", "Balmy", "Hot", "Sweltering", "Scorching"
};

app.MapGet("/weatherforecast", () =>
{
    var forecast = Enumerable.Range(1, 5).Select(index =>
        new WeatherForecast
        (
            DateOnly.FromDateTime(DateTime.Now.AddDays(index)),
            Random.Shared.Next(-20, 55),
            summaries[Random.Shared.Next(summaries.Length)]
        ))
        .ToArray();

    return forecast;
})
.WithName("GetWeatherForecast");

app.MapGet("/item-types", async (DaprClient client) =>
{
    var res = await client.InvokeMethodAsync<List<ItemTypeDto>>(HttpMethod.Get, "product-app", "v1-get-item-types");

    var curActivity = Activity.Current;
    curActivity?.AddBaggage("method-name", "item-types");

    return res;
})
.WithName("GetItemTypes");

app.MapPost("/ping", async (DaprClient client) =>
{
    await client.PublishEventAsync("pubsub", "pinged", new { Id = Guid.NewGuid() });
    return Results.Ok();
});

app.MapPost("/pong", [Topic("pubsub", "ponged")] async (Pong pong) =>
{
    Console.WriteLine($"Pong received: {pong.Id}");
    return Results.Ok();
});

app.Run();

internal record WeatherForecast(DateOnly Date, int TemperatureC, string? Summary)
{
    public int TemperatureF => 32 + (int)(TemperatureC / 0.5556);
}

internal record ItemTypeDto(string Name, int ItemType, float Price, string Image);
internal record Pong(Guid Id);
