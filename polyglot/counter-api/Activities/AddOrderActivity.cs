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
        
        var order = await Order.From(input, _itemGateway);
        order.Id = new Guid(orderId);
        await _orderRepository.AddAsync(order);
        
        var @events = new IDomainEvent[order.DomainEvents.Count];
        order.DomainEvents.CopyTo(@events);
        order.DomainEvents.Clear();

        foreach (var @event in @events)
        {
            switch (@event)
            {
                case BaristaOrderIn baristaOrderInEvent:
                    await _daprClient.PublishEventAsync(
                        "baristapubsub",
                        nameof(BaristaOrdered).ToLowerInvariant(),
                        baristaOrderInEvent);
                    break;
                case KitchenOrderIn kitchenOrderInEvent:
                    await _daprClient.PublishEventAsync(
                        "kitchenpubsub",
                        nameof(KitchenOrdered).ToLowerInvariant(),
                        kitchenOrderInEvent);
                    break;
            }
        }

        return new PlaceOrderResult(true);
    }
}