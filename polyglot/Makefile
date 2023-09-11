include .env
export

run-product:
	cd product-api && spin build && spin up --listen 0.0.0.0:5001 && cd -
.PHONY: run-product

run-product-dapr:
	cd product-api && \
	spin build && \
	dapr run \
    --app-id productapi \
    --app-port 5001 \
    --resources-path ../components \
    --config ../components/daprConfig.yaml \
    -- spin up --listen 0.0.0.0:5001 && \
	cd -
.PHONY: run-product-dapr

run-counter-dapr:
	dapr run \
    --app-id counterapi \
    --app-port 5002 \
    --dapr-http-port 3500 \
    --resources-path components \
    --config components/daprConfig.yaml \
    -- dotnet run --project counter-api/counter-api.csproj \
	cd -
.PHONY: run-counter-dapr

run-barista-dapr:
	cd barista-api && \
	spin build && \
	dapr run \
    --app-id baristaapi \
    --app-port 5003 \
    --resources-path ../components \
    --config ../components/daprConfig.yaml \
    -- spin up --listen 0.0.0.0:5003 --env DAPR_URL=${DAPR_URL} && \
	cd -
.PHONY: run-barista-dapr

run-kitchen-dapr:
	cd kitchen-api && \
	spin build && \
	dapr run \
    --app-id kitchenaapi \
    --app-port 5004 \
    --resources-path ../components \
    --config ../components/daprConfig.yaml \
    -- spin up --listen 0.0.0.0:5004 --env DAPR_URL=${DAPR_URL} && \
	cd -
.PHONY: run-kitchen-dapr

docker-build-product:
    cd product-api && \
    docker buildx build -f Dockerfile --platform wasi/wasm,linux/amd64,linux/arm64 -t ghcr.io/thangchung/dapr-labs/product-api-spin:1.0.0 . --push && \
    cd -
.PHONY: docker-build-product

docker-build-barista:
    cd barista-api && \
    docker buildx build -f Dockerfile --platform wasi/wasm,linux/amd64,linux/arm64 -t ghcr.io/thangchung/dapr-labs/barista-api-spin:1.0.0 . --push && \
    cd -
.PHONY: docker-build-barista

docker-build-kitchen:
    cd kitchen-api && \
    docker buildx build -f Dockerfile --platform wasi/wasm,linux/amd64,linux/arm64 -t ghcr.io/thangchung/dapr-labs/kitchen-api-spin:1.0.0 . --push && \
    cd -
.PHONY: docker-build-kitchen
