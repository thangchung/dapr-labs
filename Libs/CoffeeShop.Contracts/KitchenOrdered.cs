namespace CoffeeShop.Contracts;

public record KitchenOrdered
{
    public Guid OrderId { get; set; }
    public Guid ItemLineId { get; set; }
    public ItemType ItemType { get; set; }
}
