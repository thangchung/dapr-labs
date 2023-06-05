var builder = WebApplication.CreateBuilder(args);
var app = builder.Build();

app.MapGet("/", () => "Hello World!");
// app.MapPost("/v1/api/orders", async () => );



app.Run();