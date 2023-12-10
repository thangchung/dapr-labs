// dotnet ef migrations add InitKitchenDb -c MainDbContext -o Infrastructure/Data/Migrations

using KitchenService.Domain;
using KitchenService.Infrastructure.Data;
using N8T.Infrastructure;
using N8T.Infrastructure.Controller;
using N8T.Infrastructure.EfCore;
using Spectre.Console;
using System.Net;
using System.Text.Json;
using CoffeeShop.Contracts;
using Dapr;
using KitchenService.UseCases;
using MediatR;

AnsiConsole.Write(new FigletText("Kitchen APIs").Color(Color.MediumPurple));

var builder = WebApplication.CreateBuilder(args);
builder.WebHost
    // .AddOTelLogs()
    .ConfigureKestrel(webBuilder =>
    {
        webBuilder.Listen(IPAddress.Any, builder.Configuration.GetValue("RestPort", 5004)); // REST
    });

builder.Services
    .AddHttpContextAccessor()
    .AddCustomMediatR(new[] {typeof(KitchenOrder)})
    .AddCustomValidators(new[] {typeof(KitchenOrder)});

builder.Services
    .AddPostgresDbContext<MainDbContext>(
        builder.Configuration.GetConnectionString("kitchendb"),
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

    var kitchenOrderedTopic = new TopicOptions
    {
        PubsubName = "kitchenpubsub",
        Name = "kitchenordered",
        DeadLetterTopic = "kitchenorderedDeadLetterTopic"
    };

    endpoints.MapPost(
        "subscribe_KitchenOrdered",
        async (KitchenOrdered @event, ISender sender) => await sender.Send(
            new PlaceKitchenOrderCommand(@event.OrderId, @event.ItemLineId, @event.ItemType))
    ).WithTopic(kitchenOrderedTopic);
});

app.Run();