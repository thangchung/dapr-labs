using CounterApi.Activities;
using CounterApi.Domain.Commands;
using CounterApi.Domain.Messages;

using Dapr.Workflow;

namespace CounterApi.Workflows;

public record PlaceOrderResult(bool Success);
public record PlaceOrderWorkflowResult(bool Success);
public record Notification(string Message);

public class PlaceOrderWorkflow : Workflow<PlaceOrderCommand, PlaceOrderWorkflowResult>
{
    public override async Task<PlaceOrderWorkflowResult> RunAsync(WorkflowContext context, PlaceOrderCommand input)
    {
        var retryOptions = new WorkflowTaskOptions
        {
            RetryPolicy = new WorkflowRetryPolicy(
                firstRetryInterval: TimeSpan.FromMinutes(1),
                backoffCoefficient: 2.0,
                maxRetryInterval: TimeSpan.FromHours(1),
                maxNumberOfAttempts: 10),
        };

        input.OrderId = new Guid(context.InstanceId);

        await context.CallActivityAsync(
            nameof(NotifyActivity),
            new Notification($"Received order {context.InstanceId}"),
            retryOptions);

        var result = await context.CallActivityAsync<PlaceOrderResult>(
            nameof(AddOrderActivity),
            input,
            retryOptions);

        if (result.Success)
        {
            try
            {
                // Pause and wait for barista & kitchen event
                context.SetCustomStatus("Waiting for barista & kitchen events");
                var baristaOrderUpdatedEvent = context.WaitForExternalEventAsync<BaristaOrderUpdated>(
                    eventName: "BaristaOrderUpdated",
                    timeout: TimeSpan.FromSeconds(30)); //todo: read from inputParams, make sure it is deterministic

                var kitchenOrderUpdatedEvent = context.WaitForExternalEventAsync<KitchenOrderUpdated>(
                    eventName: "KitchenOrderUpdated",
                    timeout: TimeSpan.FromSeconds(30)); //todo: read from inputParams, make sure it is deterministic

                await Task.WhenAll(baristaOrderUpdatedEvent, kitchenOrderUpdatedEvent);

                var baristaOrderUpdatedResult = await baristaOrderUpdatedEvent;
                var kitchenOrderUpdatedResult = await kitchenOrderUpdatedEvent;

                // merge items
                foreach (var temp in kitchenOrderUpdatedResult.ItemLines)
                {
                    baristaOrderUpdatedResult.ItemLines.Add(temp);
                }

                await context.CallActivityAsync(
                    nameof(UpdateOrderActivity),
                    baristaOrderUpdatedResult,
                    retryOptions);

                context.SetCustomStatus($"Order {context.InstanceId} completed.");

                await context.CallActivityAsync(
                    nameof(NotifyActivity),
                    new Notification($"Completed: order {context.InstanceId}"),
                    retryOptions);
            }
            catch (TaskCanceledException)
            {
                // todo refund money
                // ...

                await context.CallActivityAsync(
                    nameof(NotifyActivity),
                    new Notification($"Failed: order {context.InstanceId} (refund money)"),
                    retryOptions);

                context.SetCustomStatus(
                    "[TaskCanceledException] Stopped order process due to time-out when called to barista & kitchen actions.");

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
                        new Notification($"Failed: order {context.InstanceId} (refund money)"),
                        retryOptions);

                    context.SetCustomStatus("[Exception] Stopped order process due to error in update actions.");

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
                new Notification($"Failed: order {context.InstanceId} (refund money)"),
                retryOptions);

            context.SetCustomStatus("Stopped order process due to place order issue.");

            return new PlaceOrderWorkflowResult(Success: false);
        }

        return new PlaceOrderWorkflowResult(Success: true);
    }
}