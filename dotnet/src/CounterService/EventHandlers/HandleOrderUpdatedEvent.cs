﻿using CounterService.Domain.DomainEvents;

namespace CounterService.EventHandlers;

public class HandleOrderUpdateEvent : N8T.Infrastructure.Events.DomainEventHandler<OrderUpdate>
{
    // private readonly IHubContext<NotificationHub, INotificationClient> _hubContext;
    //
    // public HandleOrderUpdateEvent(IHubContext<NotificationHub, INotificationClient> hubContext)
    // {
    //     _hubContext = hubContext;
    // }

    public override async Task HandleEvent(OrderUpdate @event, CancellationToken cancellationToken)
    {
        ArgumentNullException.ThrowIfNull(@event);

        var message = $"[{@event.GetType().Name}] {@event.OrderId}-{@event.ItemLineId}-{@event.OrderStatus}";
        Console.WriteLine(message);
        // await _hubContext.Clients.All.SendMessage(message);
    }
}
