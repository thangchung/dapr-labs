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
        var resource = new SpinAppResource(name, workingDirectory);

        return builder.AddResource(resource)
            .BuildAppCommand(workingDirectory)
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
        this IResourceBuilder<SpinAppResource> builder, string workingDirectory)
    {
        builder.WithCommand(
            name: "build",
            displayName: "Build Spin App",
            executeCommand: context => OnRunReBuildSpinAppCommandAsync(builder, context, workingDirectory),
            iconName: "BuildingFactory",
            iconVariant: IconVariant.Filled);

        return builder;
    }

    private static async Task<ExecuteCommandResult> OnRunReBuildSpinAppCommandAsync(
        IResourceBuilder<SpinAppResource> builder,
        ExecuteCommandContext context,
        string workingDirectory)
    {
        var logger = context.ServiceProvider.GetRequiredService<ResourceLoggerService>().GetLogger(builder.Resource);
        var notificationService = context.ServiceProvider.GetRequiredService<ResourceNotificationService>();

        await Task.Run(async () =>
        {
            var cmd = Cli.Wrap("spin").WithArguments(["build"]).WithWorkingDirectory(workingDirectory);
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
                        logger.LogInformation("{ResourceName} stdout: {StdOut}", builder.Resource.Name, stdOut.Text);
                        break;
                    case StandardErrorCommandEvent stdErr:
                        logger.LogInformation("{ResourceName} stderr: {StdErr}", builder.Resource.Name, stdErr.Text);
                        break;
                }
            }
        });

        return CommandResults.Success();
    }
}
