using CounterApi.Workflows;

using Dapr.Workflow;

namespace CounterApi.Activities;

public class NotifyActivity(ILoggerFactory loggerFactory) : WorkflowActivity<Notification, object?>
{
    private readonly ILogger _logger = loggerFactory.CreateLogger<NotifyActivity>();

    public override Task<object?> RunAsync(WorkflowActivityContext context, Notification input)
    {
        ArgumentNullException.ThrowIfNull(input);

        _logger.LogInformation(input.Message);

        //todo: inject DaprClient to publish event for Notification API

        return Task.FromResult<object?>(null);
    }
}