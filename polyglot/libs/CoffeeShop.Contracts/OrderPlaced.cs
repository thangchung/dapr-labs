namespace CoffeeShop.Contracts;

public class OrderItemDto
{
    public Guid ItemLineId { get; set; }
    public ItemType ItemType { get; set; }
}

public record BaristaOrderPlaced
{
    public Guid OrderId { get; set; }
    public List<OrderItemDto> ItemLines { get; set; } = new();
}

public record KitchenOrderPlaced
{
    public Guid OrderId { get; set; }
    public List<OrderItemDto> ItemLines { get; set; } = new();
}

