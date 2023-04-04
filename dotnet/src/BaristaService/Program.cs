// dotnet ef migrations add InitBaristaDb -c MainDbContext -o Infrastructure/Data/Migrations

using BaristaService.Domain;
using N8T.Infrastructure;
using N8T.Infrastructure.Controller;
using N8T.Infrastructure.EfCore;
using N8T.Infrastructure.OTel;
using Spectre.Console;
using System.Net;
using System.Text.Json;
using BaristaService.Infrastructure.Data;
using BaristaService.UseCases;
using CoffeeShop.Contracts;
using Dapr;
using MediatR;

AnsiConsole.Write(new FigletText("Barista APIs").Color(Color.MediumPurple));

var builder = WebApplication.CreateBuilder(args);
builder.WebHost
    // .AddOTelLogs()
    .ConfigureKestrel(webBuilder =>
    {
        webBuilder.Listen(IPAddress.Any, builder.Configuration.GetValue("RestPort", 5003)); // REST
    });

builder.Services
    .AddHttpContextAccessor()
    .AddCustomMediatR(new[] {typeof(BaristaItem)})
    .AddCustomValidators(new[] {typeof(BaristaItem)});

builder.Services
    .AddPostgresDbContext<MainDbContext>(
        builder.Configuration.GetConnectionString("baristadb"),
        null,
        svc => svc.AddRepository(typeof(Repository<>)))
    .AddDatabaseDeveloperPageExceptionFilter();

// builder.Services
//     .AddOTelTracing(builder.Configuration)
//     .AddOTelMetrics(builder.Configuration);

builder.Services.AddDaprClient();
builder.Services.AddSingleton(new JsonSerializerOptions()
{
    PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
    PropertyNameCaseInsensitive = true,
});

var app = builder.Build();

if (!app.Environment.IsDevelopment())
{
    app.UseExceptionHandler("/Error");
}

app.MapGet("/error", () => Results.Problem("An error occurred.", statusCode: 500))
    .ExcludeFromDescription();

app.UseMiddleware<ExceptionMiddleware>();

app.UseRouting();

app.UseCloudEvents();

//app.UseAuthorization();

await app.DoDbMigrationAsync(app.Logger);

app.UseEndpoints(endpoints =>
{
    endpoints.MapSubscribeHandler();

    var baristaOrderedTopic = new TopicOptions
    {
        PubsubName = "baristapubsub",
        Name = "baristaordered",
        DeadLetterTopic = "baristaorderedDeadLetterTopic"
    };

    endpoints.MapPost(
        "subscribe_BaristaOrdered",
        async (BaristaOrdered @event, ISender sender) => await sender.Send(
            new PlaceBaristaOrderCommand(@event.OrderId, @event.ItemLineId, @event.ItemType))
    ).WithTopic(baristaOrderedTopic);
});

app.Run();