using CoffeeShop.Contracts;
using CounterService.Domain;
using CounterService.Domain.DomainEvents;
using FluentValidation;
using MediatR;
using N8T.Core.Domain;
using N8T.Core.Repository;

namespace CounterService.UseCases;

public record OrderUpdatedCommand(Guid OrderId, Guid ItemLineId, string Name, ItemType ItemType, DateTime TimeIn,
    string MadeBy, DateTime TimeUp) : IRequest<IResult>;

internal class OrderUpdatedCommandValidator : AbstractValidator<OrderUpdatedCommand>
{
}

internal class OrderUpdatedCommandHandler : IRequestHandler<OrderUpdatedCommand, IResult>
{
    private readonly IRepository<Order> _orderRepository;
    private readonly IPublisher _publisher;
    private readonly ILogger<OrderUpdatedCommandHandler> _logger;

    public OrderUpdatedCommandHandler(IRepository<Order> orderRepository, IPublisher publisher, ILogger<OrderUpdatedCommandHandler> logger)
    {
        _orderRepository = orderRepository;
        _publisher = publisher;
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

        await order.RelayAndPublishEvents(_publisher, cancellationToken: cancellationToken);

        return Results.Ok();
    }
}
