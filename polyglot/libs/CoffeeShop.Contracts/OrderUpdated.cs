namespace CoffeeShop.Contracts;

public record BaristaOrderUpdated
{
    public Guid OrderId { get; set; }
    public List<OrderItemDto> ItemLines { get; set; } = new();
}

public record KitchenOrderUpdated
{
    public Guid OrderId { get; set; }
    public List<OrderItemDto> ItemLines { get; set; } = new();
}
