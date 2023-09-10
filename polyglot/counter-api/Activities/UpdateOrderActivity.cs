using CounterApi.Domain;
using CounterApi.Domain.DomainEvents;
using CounterApi.Domain.Dtos;
using CounterApi.Domain.Messages;

using Dapr.Client;
using Dapr.Workflow;
using Newtonsoft.Json;

namespace CounterApi.Activities;

public class UpdateOrderActivity(DaprClient daprClient, IItemGateway itemGateway, ILogger<UpdateOrderActivity> logger) : WorkflowActivity<BaristaOrderUpdated, object?>
{
    public override async Task<object?> RunAsync(WorkflowActivityContext context, BaristaOrderUpdated input)
    {
        ArgumentNullException.ThrowIfNull(context);
        ArgumentNullException.ThrowIfNull(input);

        logger.LogInformation("Order is {OrderId}", input.OrderId);
        var orderState = await daprClient.GetStateEntryAsync<OrderDto>("statestore", $"order-{input.OrderId}");
        logger.LogInformation("orderState: {orderState}", JsonConvert.SerializeObject(orderState));

        if (orderState.Value is not null)
        {
            var order = await Order.FromDto(orderState.Value, itemGateway);
            logger.LogInformation("order: {order}", JsonConvert.SerializeObject(order));

            foreach (var lineItem in input.ItemLines)
            {
                logger.LogInformation("Order is {OrderId}, updated BaristaOrderItem={BaristaOrderItemId}", input.OrderId,
                        lineItem.ItemLineId);

                order = order.Apply(new OrderUp(lineItem.ItemLineId));
            }

            logger.LogInformation("order-updated: {order}", JsonConvert.SerializeObject(order));
            var dto = Order.ToDto(order);
            orderState.Value = dto;
            var result = await orderState.TrySaveAsync();
            logger.LogInformation("Order status {OrderStatus}", order.OrderStatus);
            logger.LogInformation("BaristaOrder updated = {IsSucceed}", result);
        }

        return Task.FromResult<object?>(null);
    }
}
