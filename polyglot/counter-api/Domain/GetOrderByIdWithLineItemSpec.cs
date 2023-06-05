using System.Linq.Expressions;
using N8T.Core.Specification;

namespace CounterApi.Domain;

public class GetOrderByIdWithLineItemSpec : SpecificationBase<Order>
{
    private readonly Guid _id;

    public GetOrderByIdWithLineItemSpec(Guid id)
    {
        AddInclude(x => x.LineItems);
        _id = id;
    }

    public override Expression<Func<Order, bool>> Criteria => x => x.Id == _id;
}
