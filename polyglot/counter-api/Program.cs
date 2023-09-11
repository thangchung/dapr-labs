using System.Text.Json;
using Dapr.Workflow;
using FluentValidation;

using CounterApi.Activities;
using CounterApi.Domain;
using CounterApi.Infrastructure.Gateways;
using CounterApi.UseCases;
using CounterApi.Workflows;
using CounterApi.Extensions;

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddDaprWorkflow(options =>
{
    options.RegisterWorkflow<PlaceOrderWorkflow>();

    options.RegisterActivity<NotifyActivity>();
    options.RegisterActivity<AddOrderActivity>();
    options.RegisterActivity<UpdateOrderActivity>();
});

builder.Services.AddHttpContextAccessor();
builder.Services.AddMediatR(cfg => cfg.RegisterServicesFromAssemblyContaining<Program>());
builder.Services.AddValidatorsFromAssemblyContaining<Program>();

builder.Services.AddEndpointsApiExplorer();
builder.Services.AddSwaggerGen();

builder.Services.AddDaprClient();
builder.Services.AddSingleton(new JsonSerializerOptions()
{
    PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
    PropertyNameCaseInsensitive = true,
});

builder.Services.AddScoped<IItemGateway, ItemDaprGateway>();

// https://github.com/dapr/dotnet-sdk/blob/master/examples/Workflow/WorkflowConsoleApp/Program.cs#L31
if (string.IsNullOrEmpty(Environment.GetEnvironmentVariable("DAPR_GRPC_PORT")))
{
    Environment.SetEnvironmentVariable("DAPR_GRPC_PORT", "50001");
}

//builder.AddOpenTelemetry();

var app = builder.Build();

if (app.Environment.IsDevelopment())
{
    app.UseSwagger();
    app.UseSwaggerUI();
}

app.UseRouting();
app.UseCloudEvents();

app.Map("/", () => Results.Redirect("/swagger"));

_ = app.MapOrderInApiRoutes()
    .MapOrderUpApiRoutes()
    .MapOrderFulfillmentApiRoutes();

// Configure the prometheus endpoint for scraping metrics
// app.MapPrometheusScrapingEndpoint();
// NOTE: This should only be exposed on an internal port!
// .RequireHost("*:9100");

app.Run();
