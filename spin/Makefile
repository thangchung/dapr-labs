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

run-counter:
	cd counter-api && spin build && spin up --listen 0.0.0.0:5002 && cd -
.PHONY: run-counter

run-counter-dapr:
	cd counter-api && \
	spin build && \
	dapr run \
    --app-id counterapi \
    --app-port 5002 \
    --resources-path ../components \
    --config ../components/daprConfig.yaml \
    -- spin up --listen 0.0.0.0:5002 && \
	cd -
.PHONY: run-counter-dapr