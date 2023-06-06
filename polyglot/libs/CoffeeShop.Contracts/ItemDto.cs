using System.Reflection;

namespace CoffeeShop.Contracts;

public class OrderItemDto
{
    public Guid ItemLineId { get; set; }
    public ItemType ItemType { get; set; }
}

public class ItemDto
{
    public string Name { get; set; }
    public decimal Price { get; set; }
    public ItemType ItemType { get; set; }
    public string Image { get; set; }
}

public class ItemTypeDto
{
    public ItemType Type { get; set; }
    public string Name { get; set; } = null!;
}