using CoffeeShop.Contracts;
using CounterApi.Activities;
using Dapr.Workflow;
using CounterApi.Domain.Commands;

namespace CounterApi.Workflows;

public record PlaceOrderResult(bool Success);
public record PlaceOrderWorkflowResult(bool Success);
public record Notification(string Message);

public class PlaceOrderWorkflow : Workflow<PlaceOrderCommand, PlaceOrderWorkflowResult>
{
    public override async Task<PlaceOrderWorkflowResult> RunAsync(WorkflowContext context, PlaceOrderCommand input)
    {
        var orderId = context.InstanceId;
        
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
                // Pause and wait for barista & kitchen event
                context.SetCustomStatus("Waiting for barista & kitchen events");
                var baristaOrderUpdatedEvent = context.WaitForExternalEventAsync<BaristaOrderUpdated>(
                    eventName: "BaristaOrderUpdated",
                    timeout: TimeSpan.FromSeconds(15));

                var kitchenOrderUpdatedEvent = context.WaitForExternalEventAsync<KitchenOrderUpdated>(
                    eventName: "KitchenOrderUpdated",
                    timeout: TimeSpan.FromSeconds(15));

                await Task.WhenAll(baristaOrderUpdatedEvent, kitchenOrderUpdatedEvent);

                var baristaOrderUpdatedResult = await baristaOrderUpdatedEvent;
                var kitchenOrderUpdatedResult = await kitchenOrderUpdatedEvent;

                await context.CallActivityAsync(
                    nameof(BaristaUpdateOrderActivity),
                    baristaOrderUpdatedResult);

                await context.CallActivityAsync(
                    nameof(KitchenUpdateOrderActivity),
                    kitchenOrderUpdatedResult);

                await context.CallActivityAsync(
                    nameof(NotifyActivity),
                    new Notification($"Completed: order {orderId}"));
            }
            catch (TaskCanceledException)
            {
                // todo refund money
                // ...

                await context.CallActivityAsync(
                    nameof(NotifyActivity),
                    new Notification($"Failed: order {orderId} (refund money)"));

                context.SetCustomStatus(
                    "Stopped order process due to time-out when called to barista & kitchen actions.");

                return new PlaceOrderWorkflowResult(Success: false);
            }
            catch (Exception ex)
            {
                if (ex.InnerException is DurableTask.Core.Exceptions.TaskFailedException)
                {
                    // todo refund money
                    // ...

                    await context.CallActivityAsync(
                        nameof(NotifyActivity),
                        new Notification($"Failed: order {orderId} (refund money)"));

                    context.SetCustomStatus("Stopped order process due to error in update actions.");

                    return new PlaceOrderWorkflowResult(Success: false);
                }
            }
        }
        else
        {
            // todo refund money
            // ...
            
            await context.CallActivityAsync(
                nameof(NotifyActivity),
                new Notification($"Failed: order {orderId} (refund money)"));
            
            context.SetCustomStatus("Stopped order process due to place order issue.");
            
            return new PlaceOrderWorkflowResult(Success: false);
        }

        return new PlaceOrderWorkflowResult(Success: true);
    }
}