using Aspire.Hosting.Dapr;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using SpinAppHost;

var builder = DistributedApplication.CreateBuilder(args);

var rabbitUser = builder.AddParameter("RabbitUser");
var rabbitPass = builder.AddParameter("RabbitPassword", true);
var rmq = builder.AddRabbitMQ("rabbitMQ", rabbitUser, rabbitPass)
                   .WithManagementPlugin()
                   .WithEndpoint("tcp", e => e.Port = 5672)
                   .WithEndpoint("management", e => e.Port = 15672);

var stateStore = builder.AddDaprStateStore("statestore");
var pubSub = builder.AddDaprPubSub(
    "pubsub",
    new DaprComponentOptions
    {
        LocalPath = Path.Combine("..", "dapr" , "components", "pubsub.yaml")
    }).WaitFor(rmq);

var productApp = builder.AddSpinApp("product-app", workingDirectory: Path.Combine("..", "test-spin"), 
    args: ["--env", $"dapr_url=http://localhost:3500"])
    .WithHttpEndpoint(name: "http", targetPort: 3000, port: 8080)
    .WithDaprSidecar()
    .WithReference(stateStore)
    .WithReference(pubSub);

var webapp01 = builder.AddProject<Projects.WebApp01>("webapp01")
    .WithDaprSidecar(o => o.WithOptions(new DaprSidecarOptions { DaprHttpPort = 3500 }))
    .WithReference(stateStore)
    .WithReference(pubSub)
    .WaitFor(productApp);

// Workaround for https://github.com/dotnet/aspire/issues/2219
if (builder.Configuration.GetValue<string>("DAPR_CLI_PATH") is { } daprCliPath)
{
    builder.Services.Configure<DaprOptions>(options =>
    {
        options.DaprPath = daprCliPath;
    });
}

builder.Build().Run();
