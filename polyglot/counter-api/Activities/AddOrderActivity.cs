using CounterApi.Domain;
using CounterApi.Domain.Commands;
using CounterApi.Domain.DomainEvents;
using CounterApi.Domain.Dtos;
using CounterApi.Domain.Messages;
using CounterApi.Domain.SharedKernel;
using CounterApi.Workflows;

using Dapr.Client;
using Dapr.Workflow;

using Newtonsoft.Json;

namespace CounterApi.Activities;

public class AddOrderActivity(DaprClient daprClient, IItemGateway itemGateway, ILoggerFactory loggerFactory)
    : WorkflowActivity<PlaceOrderCommand, PlaceOrderResult?>
{
    private readonly ILogger _logger = loggerFactory.CreateLogger<NotifyActivity>();

    public override async Task<PlaceOrderResult?> RunAsync(WorkflowActivityContext context, PlaceOrderCommand input)
    {
        ArgumentNullException.ThrowIfNull(context);
        ArgumentNullException.ThrowIfNull(input);

        _logger.LogInformation("[AddOrderActivity] input={AddOrderActivity-input}", JsonConvert.SerializeObject(input));
        var orderId = context.InstanceId;

        _logger.LogInformation("Run AddOrderActivity with orderId={orderId}", orderId);
        var order = await Order.From(input, itemGateway);
        order.Id = new Guid(orderId); //todo: not good
        _logger.LogInformation("Order={order}", JsonConvert.SerializeObject(order));

        // map domain object to dto
        var dto = Order.ToDto(order);
        dto.OrderStatus = OrderStatus.IN_PROGRESS;

        await daprClient.SaveStateAsync("statestore", $"order-{order.Id}", dto);

        // save the order list
        var orderListState = await daprClient.GetStateEntryAsync<List<Guid>>("statestore", "order-list");
        if (orderListState.Value == null)
        {
            await daprClient.SaveStateAsync("statestore", "order-list", new List<Guid> { order.Id });
            _logger.LogInformation("orderListState inserted");
        }
        else
        {
            orderListState.Value.Add(order.Id);
            var result = await orderListState.TrySaveAsync();
            _logger.LogInformation("orderListState updated = {IsSucceed}", result);
        }

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
                            ItemLines = new List<OrderItemLineDto>
                            {
                                new(baristaOrderInEvent.ItemLineId, baristaOrderInEvent.ItemType, ItemStatus.IN_PROGRESS)
                            }
                        });
                    }
                    else
                    {
                        baristaEvents[baristaOrderInEvent.OrderId].ItemLines.Add(
                            new OrderItemLineDto(baristaOrderInEvent.ItemLineId, baristaOrderInEvent.ItemType, ItemStatus.IN_PROGRESS));
                    }

                    break;
                case KitchenOrderIn kitchenOrderInEvent:
                    if (!kitchenEvents.TryGetValue(kitchenOrderInEvent.OrderId, out _))
                    {
                        kitchenEvents.Add(kitchenOrderInEvent.OrderId, new KitchenOrderPlaced
                        {
                            OrderId = kitchenOrderInEvent.OrderId,
                            ItemLines = new List<OrderItemLineDto>
                            {
                                new(kitchenOrderInEvent.ItemLineId, kitchenOrderInEvent.ItemType, ItemStatus.IN_PROGRESS)
                            }
                        });
                    }
                    else
                    {
                        kitchenEvents[kitchenOrderInEvent.OrderId].ItemLines.Add(
                            new OrderItemLineDto(kitchenOrderInEvent.ItemLineId, kitchenOrderInEvent.ItemType, ItemStatus.IN_PROGRESS));
                    }

                    break;
            }
        }

        if (baristaEvents.Count > 0)
        {
            foreach (var @event in baristaEvents)
            {
                await daprClient.PublishEventAsync(
                    "baristapubsub",
                    nameof(BaristaOrderPlaced).ToLowerInvariant(),
                    @event.Value);
            }
        }

        if (kitchenEvents.Count > 0)
        {
            foreach (var @event in kitchenEvents)
            {
                await daprClient.PublishEventAsync(
                    "kitchenpubsub",
                    nameof(KitchenOrderPlaced).ToLowerInvariant(),
                    @event.Value);
            }
        }

        return new PlaceOrderResult(true);
    }
}