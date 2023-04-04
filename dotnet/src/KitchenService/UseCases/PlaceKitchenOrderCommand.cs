using CoffeeShop.Contracts;
using FluentValidation;
using KitchenService.Domain;
using MediatR;
using N8T.Core.Domain;
using N8T.Core.Repository;
using Newtonsoft.Json;

namespace KitchenService.UseCases;

public record PlaceKitchenOrderCommand(Guid OrderId, Guid ItemLineId, ItemType ItemType) : IRequest<IResult>;

internal class PlaceKitchenOrderCommandValidator : AbstractValidator<PlaceKitchenOrderCommand>
{
}

public class PlaceKitchenOrderCommandHandler : IRequestHandler<PlaceKitchenOrderCommand, IResult>
{
    private readonly IRepository<KitchenOrder> _kitchenOrderRepository;
    private readonly IPublisher _publisher;
    private readonly ILogger<PlaceKitchenOrderCommandHandler> _logger;

    public PlaceKitchenOrderCommandHandler(IRepository<KitchenOrder> kitchenOrderRepository, IPublisher publisher, ILogger<PlaceKitchenOrderCommandHandler> logger)
    {
        _kitchenOrderRepository = kitchenOrderRepository;
        _publisher = publisher;
        _logger = logger;
    }
    
    public async Task<IResult> Handle(PlaceKitchenOrderCommand request, CancellationToken cancellationToken)
    {
        ArgumentNullException.ThrowIfNull(request);
        
        _logger.LogInformation("Order info: {OrderInfo}", JsonConvert.SerializeObject(request));
        
        var kitchenOrder = KitchenOrder.From(request.OrderId, request.ItemType, DateTime.UtcNow);

        await Task.Delay(CalculateDelay(request.ItemType), cancellationToken);

        kitchenOrder.SetTimeUp(request.ItemLineId, DateTime.UtcNow);

        await _kitchenOrderRepository.AddAsync(kitchenOrder, cancellationToken: cancellationToken);

        await kitchenOrder.RelayAndPublishEvents(_publisher, cancellationToken: cancellationToken);

        return Results.Ok();
    }
    
    private static TimeSpan CalculateDelay(ItemType itemType)
    {
        return itemType switch
        {
            ItemType.CROISSANT => TimeSpan.FromSeconds(7),
            ItemType.CROISSANT_CHOCOLATE => TimeSpan.FromSeconds(7),
            ItemType.CAKEPOP => TimeSpan.FromSeconds(5),
            ItemType.MUFFIN => TimeSpan.FromSeconds(7),
            _ => TimeSpan.FromSeconds(3)
        };
    }
}