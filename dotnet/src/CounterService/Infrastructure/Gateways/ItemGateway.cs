using CoffeeShop.Contracts;
using CounterService.Domain;
using Dapr.Client;

namespace CounterService.Infrastructure.Gateways;

public class ItemRestGateway : IItemGateway
{
    private readonly IHttpClientFactory _httpClientFactory;
    private readonly DaprClient _daprClient;
    private readonly IConfiguration _config;
    private readonly ILogger<ItemRestGateway> _logger;

    public ItemRestGateway(IHttpClientFactory httpClientFactory, DaprClient daprClient, 
            IConfiguration config, ILogger<ItemRestGateway> logger)
    {
        _httpClientFactory = httpClientFactory;
        _daprClient = daprClient;
        _config = config;
        _logger = logger;
    }
    
    public async Task<IEnumerable<ItemDto>> GetItemsByType(ItemType[] itemTypes)
    {
        _logger.LogInformation("Start to call GetItemsByIdsAsync in Product Api");
        
        var httpClient = _httpClientFactory.CreateClient();
        httpClient.BaseAddress = new Uri(_config.GetValue<string>("ProductApiUri", "http://localhost:5001")!);

        var httpResponseMessage = await _daprClient.InvokeMethodAsync<List<ItemDto>>(HttpMethod.Get, "productservice", "v1-get-item-types");
        
        _logger.LogInformation("Can get {Count} items", httpResponseMessage?.Count);
        return httpResponseMessage ?? new List<ItemDto>();
    }
}
