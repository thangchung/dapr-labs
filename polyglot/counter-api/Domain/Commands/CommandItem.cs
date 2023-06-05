using CoffeeShop.Contracts;

namespace CounterApi.Domain.Commands;

// Represent the individual line items in a PlaceOrderCommand
public class CommandItem
{
    public ItemType ItemType { get; set; }
}
