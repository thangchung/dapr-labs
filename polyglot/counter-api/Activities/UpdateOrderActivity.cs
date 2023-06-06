using CoffeeShop.Contracts;
using CounterApi.Domain;
using CounterApi.Domain.DomainEvents;
using Dapr.Workflow;
using N8T.Core.Repository;

namespace CounterApi.Activities;

public class BaristaUpdateOrderActivity : WorkflowActivity<BaristaOrderUpdated, object?>
{
    private readonly IRepository<Order> _orderRepository;
    private readonly ILogger<BaristaUpdateOrderActivity> _logger;

    public BaristaUpdateOrderActivity(IRepository<Order> orderRepository, ILogger<BaristaUpdateOrderActivity> logger)
    {
        _orderRepository = orderRepository;
        _logger = logger;
    }
    
    public override async Task<object?> RunAsync(WorkflowActivityContext context, BaristaOrderUpdated input)
    {
        ArgumentNullException.ThrowIfNull(context);
        ArgumentNullException.ThrowIfNull(input);
        
        _logger.LogInformation("Order is {OrderId}", input.OrderId);
        
        var spec = new GetOrderByIdWithLineItemSpec(input.OrderId);
        var order = await _orderRepository.FindOneAsync(spec);

        foreach (var lineItem in input.ItemLines)
        {
            _logger.LogInformation("Order is {OrderId}, updated BaristaOrderItem={BaristaOrderItemId}", input.OrderId,
                lineItem.ItemLineId);
            _ = order.Apply(new OrderUp(lineItem.ItemLineId));
        }
        
        await _orderRepository.EditAsync(order);
        
        return Task.FromResult<object?>(null);
    }
}

public class KitchenUpdateOrderActivity : WorkflowActivity<KitchenOrderUpdated, object?>
{
    private readonly IRepository<Order> _orderRepository;
    private readonly ILogger<KitchenUpdateOrderActivity> _logger;

    public KitchenUpdateOrderActivity(IRepository<Order> orderRepository, ILogger<KitchenUpdateOrderActivity> logger)
    {
        _orderRepository = orderRepository;
        _logger = logger;
    }
    
    public override async Task<object?> RunAsync(WorkflowActivityContext context, KitchenOrderUpdated input)
    {
        ArgumentNullException.ThrowIfNull(context);
        ArgumentNullException.ThrowIfNull(input);
        
        _logger.LogInformation("Order is {OrderId}", input.OrderId);
        
        var spec = new GetOrderByIdWithLineItemSpec(input.OrderId);
        var order = await _orderRepository.FindOneAsync(spec);

        foreach (var lineItem in input.ItemLines)
        {
            _logger.LogInformation("Order is {OrderId}, updated KitchenOrderItem={KitchenOrderItemId}", input.OrderId,
                lineItem.ItemLineId);

            _ = order.Apply(new OrderUp(lineItem.ItemLineId));
        }

        await _orderRepository.EditAsync(order);
        
        return Task.FromResult<object?>(null);
    }
}