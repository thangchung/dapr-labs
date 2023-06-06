namespace CoffeeShop.Contracts;

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

