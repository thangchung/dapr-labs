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
