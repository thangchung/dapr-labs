using Dapr.Client;
using KitchenService.Domain.DomainEvents;
using MediatR;
using N8T.Core.Domain;
using Newtonsoft.Json;

namespace KitchenService.Infrastructure;

internal class EventDispatcher : INotificationHandler<EventWrapper>
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
        _logger.LogInformation("[KitchenService] Event Dispatcher: {EventInfo}", JsonConvert.SerializeObject(@eventWrapper.Event));
        
        if (@eventWrapper.Event is KitchenOrderUp kitchenOrderUpEvent)
        {
            await _daprClient.PublishEventAsync(
                "orderuppubsub",
                "orderup",
                kitchenOrderUpEvent,
                cancellationToken);
        }
    }
}
