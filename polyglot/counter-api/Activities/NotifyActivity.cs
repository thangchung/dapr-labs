using CounterApi.Workflows;
using Dapr.Workflow;

namespace CounterApi.Activities;

public class NotifyActivity : WorkflowActivity<Notification, object?>
{
    private readonly ILogger _logger;

    public NotifyActivity(ILoggerFactory loggerFactory)
    {
        _logger = loggerFactory.CreateLogger<NotifyActivity>();
    }
    
    public override Task<object?> RunAsync(WorkflowActivityContext context, Notification input)
    {
        _logger.LogInformation(input.Message);
        
        //todo: inject DaprClient to publish event
        
        return Task.FromResult<object?>(null);
    }
}