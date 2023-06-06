using CoffeeShop.Contracts;
using CounterApi.Domain;
using CounterApi.Domain.Commands;
using CounterApi.Domain.DomainEvents;
using CounterApi.Workflows;
using Dapr.Client;
using Dapr.Workflow;
using N8T.Core.Domain;
using N8T.Core.Repository;

namespace CounterApi.Activities;

public class AddOrderActivity : WorkflowActivity<PlaceOrderCommand, PlaceOrderResult?>
{
    private readonly IRepository<Order> _orderRepository;
    private readonly IItemGateway _itemGateway;
    private readonly DaprClient _daprClient;
    private readonly ILogger _logger;

    public AddOrderActivity(
        IRepository<Order> orderRepository, 
        IItemGateway itemGateway, 
        DaprClient daprClient,
        ILoggerFactory loggerFactory)
    {
        _orderRepository = orderRepository;
        _itemGateway = itemGateway;
        _daprClient = daprClient;
        _logger = loggerFactory.CreateLogger<NotifyActivity>();
    }

    public override async Task<PlaceOrderResult?> RunAsync(WorkflowActivityContext context, PlaceOrderCommand input)
    {
        var orderId = context.InstanceId;

        _logger.LogInformation($"Run AddOrderActivity with orderId={orderId}");

        var order = await Order.From(input, _itemGateway);// todo: we might refactor to use DaprClient directly
        order.Id = new Guid(orderId); //todo: not good
        await _orderRepository.AddAsync(order);

        var @events = new IDomainEvent[order.DomainEvents.Count];
        order.DomainEvents.CopyTo(@events);
        order.DomainEvents.Clear();

        var baristaEvents = new Dictionary<Guid, BaristaOrderPlaced>();
        var kitchenEvents = new Dictionary<Guid, KitchenOrderPlaced>();
        foreach (var @event in @events)
        {
            switch (@event)
            {
                case BaristaOrderIn baristaOrderInEvent:
                    if (!baristaEvents.TryGetValue(baristaOrderInEvent.OrderId, out _))
                    {
                        baristaEvents.Add(baristaOrderInEvent.OrderId, new BaristaOrderPlaced
                        {
                            OrderId = baristaOrderInEvent.OrderId,
                            ItemLines = new List<OrderItemDto>
                            {
                                new()
                                {
                                    ItemLineId = baristaOrderInEvent.ItemLineId, ItemType = baristaOrderInEvent.ItemType
                                }
                            }
                        });
                    }
                    else
                    {
                        baristaEvents[baristaOrderInEvent.OrderId].ItemLines.Add(new OrderItemDto
                        {
                            ItemLineId = baristaOrderInEvent.ItemLineId, ItemType = baristaOrderInEvent.ItemType
                        });
                    }

                    break;
                case KitchenOrderIn kitchenOrderInEvent:
                    if (!kitchenEvents.TryGetValue(kitchenOrderInEvent.OrderId, out _))
                    {
                        kitchenEvents.Add(kitchenOrderInEvent.OrderId, new KitchenOrderPlaced
                        {
                            OrderId = kitchenOrderInEvent.OrderId,
                            ItemLines = new List<OrderItemDto>
                            {
                                new()
                                {
                                    ItemLineId = kitchenOrderInEvent.ItemLineId, ItemType = kitchenOrderInEvent.ItemType
                                }
                            }
                        });
                    }
                    else
                    {
                        kitchenEvents[kitchenOrderInEvent.OrderId].ItemLines.Add(new OrderItemDto
                        {
                            ItemLineId = kitchenOrderInEvent.ItemLineId, ItemType = kitchenOrderInEvent.ItemType
                        });
                    }

                    break;
            }
        }

        if (baristaEvents.Count > 0)
        {
            foreach (var @event in baristaEvents)
            {
                await _daprClient.PublishEventAsync(
                    "baristapubsub",
                    nameof(BaristaOrderPlaced).ToLowerInvariant(),
                    @event.Value);
            }
        }

        if (kitchenEvents.Count > 0)
        {
            foreach (var @event in kitchenEvents)
            {
                await _daprClient.PublishEventAsync(
                    "kitchenpubsub",
                    nameof(KitchenOrderPlaced).ToLowerInvariant(),
                    @event.Value);
            }
        }

        return new PlaceOrderResult(true);
    }
}