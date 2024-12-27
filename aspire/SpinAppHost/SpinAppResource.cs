using CliWrap.EventStream;
using CliWrap;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace SpinAppHost;

internal class SpinAppResource(string name, string workingDirectory) 
    : ExecutableResource(name, "spin", workingDirectory)
{
}

internal static class SpinAppExtensions
{
    public static IResourceBuilder<SpinAppResource> AddSpinApp(
        this IDistributedApplicationBuilder builder, string name, string command = "up", string workingDirectory = "", string[]? args = null)
    {
        // builder.Services.TryAddLifecycleHook<TestResourceLifecycleHook>();

        var resource = new SpinAppResource(name, workingDirectory);

        return builder.AddResource(resource)
            .BuildAppCommand()
            .WithArgs(context =>
            {
                context.Args.Add(command);

                if (args is not null)
                {
                    foreach (var arg in args)
                    {
                        context.Args.Add(arg);
                    }
                }
            })
            .WithOtlpExporter()
            .ExcludeFromManifest();
    }

    public static IResourceBuilder<SpinAppResource> BuildAppCommand(
        this IResourceBuilder<SpinAppResource> builder)
    {
        builder.WithCommand(
            name: "build",
            displayName: "Build App",
            executeCommand: context => OnRunClearCacheCommandAsync(builder, context),
            iconName: "AnimalRabbitOff",
            iconVariant: IconVariant.Filled);

        return builder;
    }

    private static async Task<ExecuteCommandResult> OnRunClearCacheCommandAsync(
        IResourceBuilder<SpinAppResource> builder,
        ExecuteCommandContext context)
    {
        var logger = context.ServiceProvider.GetRequiredService<ResourceLoggerService>().GetLogger(builder.Resource);
        var notificationService = context.ServiceProvider.GetRequiredService<ResourceNotificationService>();

        var path = Path.Combine("..", "test-spin");

        await Task.Run(async () =>
        {
            var cmd = Cli.Wrap("spin").WithArguments(["build"]).WithWorkingDirectory(path);
            var cmdEvents = cmd.ListenAsync();

            await foreach (var cmdEvent in cmdEvents)
            {
                switch (cmdEvent)
                {
                    case StartedCommandEvent:
                        await notificationService.PublishUpdateAsync(builder.Resource, state => state with { State = "Running" });
                        break;
                    case ExitedCommandEvent:
                        await notificationService.PublishUpdateAsync(builder.Resource, state => state with { State = "Finished" });
                        break;
                    case StandardOutputCommandEvent stdOut:
                        logger.LogInformation("External container {ResourceName} stdout: {StdOut}", builder.Resource.Name, stdOut.Text);
                        break;
                    case StandardErrorCommandEvent stdErr:
                        logger.LogInformation("External container {ResourceName} stderr: {StdErr}", builder.Resource.Name, stdErr.Text);
                        break;
                }
            }
        });

        return CommandResults.Success();
    }
}

//internal sealed class TestResourceLifecycleHook(ResourceNotificationService notificationService) : IDistributedApplicationLifecycleHook
//{
//    public async Task BeforeStartAsync(DistributedApplicationModel appModel, CancellationToken cancellationToken)
//    {
//        foreach (var resource in appModel.Resources.OfType<SpinAppResource>())
//        {
//            //Task.Run(
//            //    async () =>
//            //    {
//            //        // await Task.Delay(TimeSpan.FromSeconds(10));

//            //        var urls = new List<string> { "http://localhost:3000" };

//            //        await notificationService.PublishUpdateAsync(
//            //            resource,
//            //            state => state with
//            //            {
//            //                State = new("Running", "success"),
//            //                Urls = [.. urls.Select(u => new UrlSnapshot(u, u, IsInternal: false))]
//            //            });
//            //    },
//            //cancellationToken);

//            var urls = new List<string> { "http://127.0.0.1:3000" };

//            await notificationService.PublishUpdateAsync(
//                resource,
//                state => state with
//                {
//                    State = new("Running", "success"),
//                    Urls = [.. urls.Select(u => new UrlSnapshot(u, u, IsInternal: false))]
//                });
//        }

//        // return Task.CompletedTask;
//    }
//}
