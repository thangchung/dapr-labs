using SpinAppHost;

var builder = DistributedApplication.CreateBuilder(args);

builder.AddSpinApp("product-app", workingDirectory: "../test-spin")
    .WithHttpEndpoint(name: "http", targetPort: 3000, port: 8080)
    .WithDaprSidecar();

builder.Build().Run();
