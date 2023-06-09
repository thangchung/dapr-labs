using CoffeeShop.Contracts;
using Dapr.Client;
using FluentValidation;
using MediatR;
using Newtonsoft.Json;

namespace CounterApi.Features;

public record OrderUpdatedCommand(Guid OrderId, List<OrderItemDto> ItemLines, bool IsBarista = true) : IRequest<IResult>;

internal class OrderUpdatedCommandValidator : AbstractValidator<OrderUpdatedCommand>
{
}

internal class OrderUpdatedCommandHandler : IRequestHandler<OrderUpdatedCommand, IResult>
{
    private readonly DaprClient _daprClient;
    private readonly ILogger<OrderUpdatedCommandHandler> _logger;

    public OrderUpdatedCommandHandler(DaprClient daprClient, ILogger<OrderUpdatedCommandHandler> logger)
    {
        _daprClient = daprClient;
        _logger = logger;
    }

    public async Task<IResult> Handle(OrderUpdatedCommand request, CancellationToken cancellationToken)
    {
        _logger.LogInformation("OrderUpdatedCommand received: {OrderUpdatedCommand}",
            JsonConvert.SerializeObject(request));

        if (request.IsBarista)
        {
            await _daprClient.RaiseWorkflowEventAsync(
                request.OrderId.ToString(),
                "dapr",
                "BaristaOrderUpdated",
                request
            );
        }
        else
        {
            await _daprClient.RaiseWorkflowEventAsync(
                request.OrderId.ToString(),
                "dapr",
                "KitchenOrderUpdated",
                request
            );
        }

        return Results.Ok();
    }
}