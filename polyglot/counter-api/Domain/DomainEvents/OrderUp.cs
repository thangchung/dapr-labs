using CoffeeShop.Contracts;
using N8T.Core.Domain;

namespace CounterApi.Domain.DomainEvents;

public class OrderUp : EventBase
{
    // OrderIn info
    public Guid ItemLineId { get; }

    public OrderUp(Guid itemLineId)
    {
        ItemLineId = itemLineId;
    }
}

public class BaristaOrderUp : OrderUp
{
    public BaristaOrderUp(Guid itemLineId) 
        : base(itemLineId)
    {
    }
}

public class KitchenOrderUp : OrderUp
{
    public KitchenOrderUp(Guid itemLineId)
        : base(itemLineId)
    {
    }
}
