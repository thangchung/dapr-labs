using CoffeeShop.Contracts;
using CounterService.Domain;
using Dapr.Client;

namespace CounterService.Infrastructure.Gateways;

public class ItemDaprGateway : IItemGateway
{
    private readonly DaprClient _daprClient;
    private readonly IConfiguration _config;
    private readonly ILogger<ItemDaprGateway> _logger;

    public ItemDaprGateway(DaprClient daprClient, IConfiguration config, ILogger<ItemDaprGateway> logger)
    {
        _daprClient = daprClient;
        _config = config;
        _logger = logger;
    }
    
    public async Task<IEnumerable<ItemDto>> GetItemsByType(ItemType[] itemTypes)
    {
        _logger.LogInformation("Start to call GetItemsByIdsAsync in Product Api");

        var productAppName = _config.GetValue<string>("ProductCatalogAppDaprName", "product-api-dapr-http");
        _logger.LogInformation("ProductCatalogAppDaprName: {0}", productAppName);
        
        var httpResponseMessage = await _daprClient.InvokeMethodAsync<List<ItemDto>>(
            HttpMethod.Get,
            productAppName, 
            "v1-get-item-types");
        
        _logger.LogInformation("Can get {Count} items", httpResponseMessage?.Count);
        return httpResponseMessage ?? new List<ItemDto>();
    }
}
