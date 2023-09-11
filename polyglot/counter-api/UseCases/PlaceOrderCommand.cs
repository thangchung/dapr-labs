using FluentValidation;
using Dapr.Client;
using MediatR;
using Newtonsoft.Json;

using CounterApi.Domain.Commands;
using CounterApi.Domain;
using CounterApi.Workflows;

namespace CounterApi.UseCases;

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

internal class PlaceOrderHandler(DaprClient daprClient, IItemGateway itemGateway, ILogger<PlaceOrderHandler> logger)
    : IRequestHandler<PlaceOrderCommand, IResult>
{
    public async Task<IResult> Handle(PlaceOrderCommand placeOrderCommand, CancellationToken cancellationToken)
    {
        ArgumentNullException.ThrowIfNull(placeOrderCommand);

        var itemTypes = new List<ItemType> { ItemType.ESPRESSO };
        var items = await itemGateway.GetItemsByType(itemTypes.ToArray());
        logger.LogInformation("[ProductAPI] Query: {JsonObject}", JsonConvert.SerializeObject(items));

        var orderId = Guid.NewGuid().ToString();
        await daprClient.StartWorkflowAsync(
            "dapr",
            nameof(PlaceOrderWorkflow),
            orderId,
            placeOrderCommand,
            cancellationToken: cancellationToken);

        return Results.Ok();
    }
}
