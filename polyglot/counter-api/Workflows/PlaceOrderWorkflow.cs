using CounterApi.Activities;
using CounterApi.Domain;
using Dapr.Workflow;
using CounterApi.Domain.Commands;

namespace CounterApi.Workflows;

public record PlaceOrderResult(bool Success);
public record Result; 
public record Notification(string Message);

public class PlaceOrderWorkflow : Workflow<PlaceOrderCommand, Result>
{
    public override async Task<Result> RunAsync(WorkflowContext context, PlaceOrderCommand input)
    {
        var orderId = context.InstanceId;
        var lineItemStatuses = new Dictionary<Guid, ItemStatus>();
        
        await context.CallActivityAsync(
            nameof(NotifyActivity),
            new Notification($"Received order {orderId}"));
        
        var result = await context.CallActivityAsync<PlaceOrderResult>(
            nameof(AddOrderActivity),
            input);

        if (result.Success)
        {
            try
            {
                // https://github.com/Azure/azure-functions-durable-extension/issues/275
                while (lineItemStatuses.Any(x => x.Value != ItemStatus.FULFILLED))
                {
                    //todo: works on it
                    // 1. update order
                    // 2. then update dictionary
                }

                // Pause and wait for barista
                /*context.SetCustomStatus("Waiting for barista");
                var orderCompletedResult = await context.WaitForExternalEventAsync<OrderCompleted>(
                    eventName: "OrderCompleted",
                    timeout: TimeSpan.FromSeconds(30));*/
                
                await context.CallActivityAsync(
                    nameof(NotifyActivity),
                    new Notification($"Completed: order {orderId}"));
            }
            catch (TaskCanceledException)
            {
                // todo refund money
                await context.CallActivityAsync(
                    nameof(NotifyActivity),
                    new Notification($"Failed: order {orderId} (refund money)"));
            }
        }
        else
        {
            // todo refund money
            await context.CallActivityAsync(
                nameof(NotifyActivity),
                new Notification($"Failed: order {orderId} (refund money)"));
        }

        return new Result();
    }
}