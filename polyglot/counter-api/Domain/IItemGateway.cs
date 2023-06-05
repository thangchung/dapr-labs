using CoffeeShop.Contracts;

namespace CounterApi.Domain;

public interface IItemGateway
{
    Task<IEnumerable<ItemDto>> GetItemsByType(ItemType[] itemTypes);
}