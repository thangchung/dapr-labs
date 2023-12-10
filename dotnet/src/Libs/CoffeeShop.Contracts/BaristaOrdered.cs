namespace CoffeeShop.Contracts;

public record BaristaOrdered
{
    public Guid OrderId { get; set; }
    public Guid ItemLineId { get; set; }
    public ItemType ItemType { get; set; }
}

