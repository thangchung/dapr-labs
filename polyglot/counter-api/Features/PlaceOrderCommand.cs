using CounterApi.Domain;
using CounterApi.Domain.Commands;
using FluentValidation;
using MediatR;
using N8T.Core.Domain;
using N8T.Core.Repository;
using CounterApi.Workflows;
using Dapr.Client;

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
    private readonly IRepository<Order> _orderRepository;
    private readonly IItemGateway _itemGateway;
    private readonly IPublisher _publisher;
    private readonly DaprClient _daprClient;

    public PlaceOrderHandler(IRepository<Order> orderRepository, IItemGateway itemGateway, IPublisher publisher, DaprClient daprClient)
    {
        _orderRepository = orderRepository;
        _itemGateway = itemGateway;
        _publisher = publisher;
        _daprClient = daprClient;
    }

    public async Task<IResult> Handle(PlaceOrderCommand placeOrderCommand, CancellationToken cancellationToken)
    {
        ArgumentNullException.ThrowIfNull(placeOrderCommand);

        var orderId = Guid.NewGuid().ToString();
        // var order = await Order.From(placeOrderCommand, _itemGateway);

        await _daprClient.StartWorkflowAsync(
            "place-order-workflow",
            nameof(PlaceOrderWorkflow),
            orderId,
            placeOrderCommand,
            cancellationToken: cancellationToken);
        
        // await _orderRepository.AddAsync(order, cancellationToken: cancellationToken);

        // await order.RelayAndPublishEvents(_publisher, cancellationToken);

        return Results.Ok();
    }
}