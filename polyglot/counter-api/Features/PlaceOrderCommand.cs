using CoffeeShop.Contracts;
using CounterApi.Domain;
using CounterApi.Domain.Commands;
using FluentValidation;
using MediatR;
using CounterApi.Workflows;
using Dapr.Client;
using Newtonsoft.Json;

namespace CounterApi.Features;

public static class OrderInRouteMapper
{
    public static IEndpointRouteBuilder MapOrderInApiRoutes(this IEndpointRouteBuilder builder)
    {
        builder.MapPost("/v1/api/orders", async (PlaceOrderCommand command, ISender sender) => await sender.Send(command));
        return builder;
    }
}

internal class OrderInValidator : AbstractValidator<PlaceOrderCommand>
{
}

internal class PlaceOrderHandler : IRequestHandler<PlaceOrderCommand, IResult>
{
    private readonly DaprClient _daprClient;
    private readonly IItemGateway _itemGateway;
    private readonly ILogger<PlaceOrderHandler> _logger;

    public PlaceOrderHandler(DaprClient daprClient, IItemGateway itemGateway, ILogger<PlaceOrderHandler> logger)
    {
        _daprClient = daprClient;
        _itemGateway = itemGateway;
        _logger = logger;
    }

    public async Task<IResult> Handle(PlaceOrderCommand placeOrderCommand, CancellationToken cancellationToken)
    {
        ArgumentNullException.ThrowIfNull(placeOrderCommand);

        var itemTypes = new List<ItemType> { ItemType.ESPRESSO };
        var items = await _itemGateway.GetItemsByType(itemTypes.ToArray());
        _logger.LogInformation("[ProductAPI] Query: {JsonObject}", JsonConvert.SerializeObject(items));
        
        var orderId = Guid.NewGuid().ToString();
        await _daprClient.StartWorkflowAsync(
            "dapr",
            nameof(PlaceOrderWorkflow),
            orderId,
            placeOrderCommand,
            cancellationToken: cancellationToken);

        return Results.Ok();
    }
}