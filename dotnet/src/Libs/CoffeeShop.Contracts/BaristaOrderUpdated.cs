namespace CoffeeShop.Contracts;

public record BaristaOrderUpdated
{
    public Guid OrderId { get; set; }
    public Guid ItemLineId { get; set; }
    public string Name { get; set; }
    public ItemType ItemType { get; set; }
    public DateTime TimeIn { get; set; }
    public string MadeBy { get; set; }
    public DateTime TimeUp { get; set; }
}
