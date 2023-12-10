using CoffeeShop.Contracts;
using CounterService.Domain.DomainEvents;
using Dapr.Client;
using MediatR;
using N8T.Core.Domain;
using Newtonsoft.Json;

namespace CounterService.Infrastructure;

public class EventDispatcher : INotificationHandler<EventWrapper>
{
    private readonly DaprClient _daprClient;
    private readonly ILogger<EventDispatcher> _logger;

    public EventDispatcher(DaprClient daprClient, ILogger<EventDispatcher> logger)
    {
        _daprClient = daprClient;
        _logger = logger;
    }

    public virtual async Task Handle(EventWrapper @eventWrapper, CancellationToken cancellationToken)
    {
        _logger.LogInformation("[CounterService] Event Dispatcher: {EventInfo}", JsonConvert.SerializeObject(@eventWrapper.Event));
        
        switch (@eventWrapper.Event)
        {
            case BaristaOrderIn baristaOrderInEvent:
                await _daprClient.PublishEventAsync(
                    "baristapubsub",
                    nameof(BaristaOrdered).ToLowerInvariant(),
                    baristaOrderInEvent,
                    cancellationToken);
                break;
            case KitchenOrderIn kitchenOrderInEvent:
                await _daprClient.PublishEventAsync(
                    "kitchenpubsub",
                    nameof(KitchenOrdered).ToLowerInvariant(),
                    kitchenOrderInEvent,
                    cancellationToken);
                break;
        }
    }
}
