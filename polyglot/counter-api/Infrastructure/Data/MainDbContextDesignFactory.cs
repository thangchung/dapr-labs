using N8T.Infrastructure.EfCore;

namespace CounterApi.Infrastructure.Data;

public class MainDbContextDesignFactory : DbContextDesignFactoryBase<MainDbContext>
{
    public MainDbContextDesignFactory() : base("counterdb")
    {
    }
}
