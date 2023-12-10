using BaristaService.Domain.DomainEvents;
using Dapr.Client;
using MediatR;
using N8T.Core.Domain;
using Newtonsoft.Json;

namespace BaristaService.Infrastructure;

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
        _logger.LogInformation("[BaristaService] Event Dispatcher: {EventInfo}", JsonConvert.SerializeObject(@eventWrapper.Event));
        
        if (@eventWrapper.Event is BaristaOrderUp baristaOrderUpEvent)
        {
            await _daprClient.PublishEventAsync(
                "orderuppubsub",
                "orderup",
                baristaOrderUpEvent,
                cancellationToken);
        }
    }
}
