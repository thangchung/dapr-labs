using CoffeeShop.Contracts;
using CounterApi.Domain;
using CounterApi.Domain.DomainEvents;
using Dapr.Client;
using FluentValidation;
using MediatR;
using N8T.Core.Domain;
using N8T.Core.Repository;

namespace CounterApi.Features;

public record OrderUpdatedCommand(Guid OrderId, Guid ItemLineId, string Name, ItemType ItemType, DateTime TimeIn,
    string MadeBy, DateTime TimeUp) : IRequest<IResult>;

internal class OrderUpdatedCommandValidator : AbstractValidator<OrderUpdatedCommand>
{
}

internal class OrderUpdatedCommandHandler : IRequestHandler<OrderUpdatedCommand, IResult>
{
    private readonly IRepository<Order> _orderRepository;
    private readonly DaprClient _daprClient;
    private readonly ILogger<OrderUpdatedCommandHandler> _logger;

    public OrderUpdatedCommandHandler(IRepository<Order> orderRepository, DaprClient daprClient, ILogger<OrderUpdatedCommandHandler> logger)
    {
        _orderRepository = orderRepository;
        _daprClient = daprClient;
        _logger = logger;
    }
    
    public async Task<IResult> Handle(OrderUpdatedCommand request, CancellationToken cancellationToken)
    {
        ArgumentNullException.ThrowIfNull(request);
        
        _logger.LogInformation("Order is {OrderId}", request.OrderId);

        var spec = new GetOrderByIdWithLineItemSpec(request.OrderId);
        var order = await _orderRepository.FindOneAsync(spec, cancellationToken);
        
        var orderUpdated = order.Apply(
            new OrderUp(
                request.OrderId, 
                request.ItemLineId, 
                request.ItemType.ToString(), 
                request.ItemType,
                request.TimeUp,
                request.MadeBy));
        
        await _orderRepository.EditAsync(orderUpdated, cancellationToken: cancellationToken);

        // await order.RelayAndPublishEvents(_publisher, cancellationToken: cancellationToken);
        var @events = new IDomainEvent[order.DomainEvents.Count];
        order.DomainEvents.CopyTo(@events);
        order.DomainEvents.Clear();

        foreach (var @event in @events)
        {
            switch (@event)
            {
                /*case OrderCompleted orderCompleted:
                    await _daprClient.RaiseWorkflowEventAsync(
                        instanceId: orderUpdated.Id.ToString(),
                        workflowComponent: "place-order-workflow",
                        eventName: "OrderCompleted",
                        orderCompleted,
                        cancellationToken);
                    break;*/
            }
        }
        
        return Results.Ok();
    }
}
