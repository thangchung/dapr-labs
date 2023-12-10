using BaristaService.Domain;
using CoffeeShop.Contracts;
using FluentValidation;
using MediatR;
using N8T.Core.Domain;
using N8T.Core.Repository;
using Newtonsoft.Json;

namespace BaristaService.UseCases;

public record PlaceBaristaOrderCommand(Guid OrderId, Guid ItemLineId, ItemType ItemType) : IRequest<IResult>;

internal class PlaceBaristaOrderCommandValidator : AbstractValidator<PlaceBaristaOrderCommand>
{
}

internal class PlaceBaristaOrderCommandHandler : IRequestHandler<PlaceBaristaOrderCommand, IResult>
{
    private readonly IRepository<BaristaItem> _baristaItemRepository;
    private readonly IPublisher _publisher;
    private readonly ILogger<PlaceBaristaOrderCommandHandler> _logger;

    public PlaceBaristaOrderCommandHandler(IRepository<BaristaItem> baristaItemRepository, IPublisher publisher, ILogger<PlaceBaristaOrderCommandHandler> logger)
    {
        _baristaItemRepository = baristaItemRepository;
        _publisher = publisher;
        _logger = logger;
    }
    
    public async Task<IResult> Handle(PlaceBaristaOrderCommand request, CancellationToken cancellationToken)
    {
        ArgumentNullException.ThrowIfNull(request);
        
        _logger.LogInformation("Order info: {OrderInfo}", JsonConvert.SerializeObject(request));
        
        var baristaItem = BaristaItem.From(request.ItemType, request.ItemType.ToString(), DateTime.UtcNow);

        await Task.Delay(CalculateDelay(request.ItemType), cancellationToken);

        _ = baristaItem.SetTimeUp(request.OrderId, request.ItemLineId, DateTime.UtcNow);

        await _baristaItemRepository.AddAsync(baristaItem, cancellationToken: cancellationToken);

        await baristaItem.RelayAndPublishEvents(_publisher, cancellationToken: cancellationToken);

        return Results.Ok();
    }
    
    private static TimeSpan CalculateDelay(ItemType itemType)
    {
        return itemType switch
        {
            ItemType.COFFEE_BLACK => TimeSpan.FromSeconds(5),
            ItemType.COFFEE_WITH_ROOM => TimeSpan.FromSeconds(5),
            ItemType.ESPRESSO => TimeSpan.FromSeconds(7),
            ItemType.ESPRESSO_DOUBLE => TimeSpan.FromSeconds(7),
            ItemType.CAPPUCCINO => TimeSpan.FromSeconds(10),
            _ => TimeSpan.FromSeconds(3)
        };
    }
}