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
    new Aspire.Hosting.Dapr.DaprComponentOptions
    {
        LocalPath = Path.Combine("..", "dapr/components/pubsub.yaml")
    }).WaitFor(rmq);

var productApp = builder.AddSpinApp("product-app", workingDirectory: "../test-spin"/*, args: ["--build"]*/)
    .WithHttpEndpoint(name: "http", targetPort: 3000, port: 8080)
    .WithDaprSidecar()
    .WithReference(stateStore)
    .WithReference(pubSub);

builder.AddProject<Projects.WebApp01>("webapp01")
    .WithDaprSidecar()
    .WithReference(stateStore)
    .WithReference(pubSub)
    .WaitFor(productApp);

builder.Build().Run();
