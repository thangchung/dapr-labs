using CoffeeShop.Contracts;
using Dapr.Workflow;

namespace CounterApi.Activities;

public class UpdateOrderActivity : WorkflowActivity<BaristaOrderUpdated, object?>
{
    public override Task<object?> RunAsync(WorkflowActivityContext context, BaristaOrderUpdated input)
    {
        throw new NotImplementedException();
    }
}