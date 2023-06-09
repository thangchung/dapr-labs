using Spectre.Console;
using System.Net;
using N8T.Infrastructure;
using ProductService.Domain;
using ProductService.UseCases;

AnsiConsole.Write(new FigletText("Product APIs").Color(Color.MediumPurple));

var builder = WebApplication.CreateBuilder(args);

builder.WebHost
    // .AddOTelLogs()
    .ConfigureKestrel(webBuilder =>
    {
        webBuilder.Listen(IPAddress.Any, builder.Configuration.GetValue("RestPort", 5001)); // REST
    });

builder.Services
    .AddHttpContextAccessor()
    .AddCustomMediatR(new[] {typeof(Item)})
    .AddCustomValidators(new[] {typeof(Item)});
    
    // .AddOTelTracing(builder.Configuration)
    // .AddOTelMetrics(builder.Configuration);

var app = builder.Build();

_ = app.MapItemTypesQueryApiRoutes()
    .MapItemsByIdsQueryApiRoutes();

app.Run();
