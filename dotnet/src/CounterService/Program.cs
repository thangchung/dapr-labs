// dotnet ef migrations add InitCounterDb -c MainDbContext -o Infrastructure/Data/Migrations

using CounterService.Domain;
using CounterService.UseCases;
using CounterService.Infrastructure.Data;
using CounterService.Infrastructure.Gateways;
using N8T.Infrastructure;
using N8T.Infrastructure.Controller;
using N8T.Infrastructure.EfCore;
using Spectre.Console;
using System.Net;
using System.Text.Json;
using CoffeeShop.Contracts;
using Dapr;
using MediatR;

AnsiConsole.Write(new FigletText("Counter APIs").Color(Color.MediumPurple));

var builder = WebApplication.CreateBuilder(args);

builder.WebHost
    // .AddOTelLogs()
    .ConfigureKestrel(webBuilder =>
    {
        webBuilder.Listen(IPAddress.Any, builder.Configuration.GetValue("RestPort", 5002)); // REST
    });

builder.Services
    .AddHttpContextAccessor()
    .AddCustomMediatR(new[] { typeof(Order) })
    .AddCustomValidators(new[] { typeof(Order) });

builder.Services
    .AddPostgresDbContext<MainDbContext>(
        builder.Configuration.GetConnectionString("counterdb"),
        null,
        svc => svc.AddRepository(typeof(Repository<>)))
    .AddDatabaseDeveloperPageExceptionFilter();

// builder.Services.AddSignalR();

// builder.Services
//     .AddOTelTracing(builder.Configuration)
//     .AddOTelMetrics(builder.Configuration);

// builder.Services.AddHttpClient();
builder.Services.AddScoped<IItemGateway, ItemDaprGateway>();
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

app.UseEndpoints(endpoints =>
    {
        endpoints.MapSubscribeHandler();

        var orderUpTopic = new TopicOptions
        {
            PubsubName = "orderuppubsub",
            Name = "orderup",
            DeadLetterTopic = "orderupDeadLetterTopic"
        };

        endpoints.MapPost(
            "subscribe_BaristaOrderUpdated",
            async (BaristaOrderUpdated @event, ISender sender) => await sender.Send(
                new OrderUpdatedCommand(
                    @event.OrderId,
                    @event.ItemLineId,
                    @event.Name,
                    @event.ItemType,
                    @event.TimeIn,
                    @event.MadeBy,
                    @event.TimeUp))
        ).WithTopic(orderUpTopic);

        endpoints.MapPost(
            "subscribe_KitchenOrderUpdated",
            async (KitchenOrderUpdated @event, ISender sender) => await sender.Send(
                new OrderUpdatedCommand(
                    @event.OrderId,
                    @event.ItemLineId,
                    @event.Name,
                    @event.ItemType,
                    @event.TimeIn,
                    @event.MadeBy,
                    @event.TimeUp))
        ).WithTopic(orderUpTopic);
    }
);

_ = app.MapOrderInApiRoutes()
    .MapOrderFulfillmentApiRoutes();

// app.MapHub<NotificationHub>("/message");

await app.DoDbMigrationAsync(app.Logger);

app.Run();