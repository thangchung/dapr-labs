#!/bin/sh

## Create a k3d cluster
while (! kubectl cluster-info ); do
  # Docker takes a few seconds to initialize
  echo "Waiting for Docker to launch..."
  k3d cluster delete wasm-cluster

  # install containerd-wasm-shims
  k3d cluster create wasm-cluster --image ghcr.io/deislabs/containerd-wasm-shims/examples/k3d:v0.9.3 -p "8081:80@loadbalancer" --agents 2
  kubectl apply -f https://github.com/deislabs/containerd-wasm-shims/raw/main/deployments/workloads/runtime.yaml
  kubectl apply -f https://github.com/deislabs/containerd-wasm-shims/raw/main/deployments/workloads/workload.yaml
  echo "waiting 5 seconds for workload to be ready"
  sleep 15
  curl -v http://127.0.0.1:8081/spin/hello
  curl -v http://127.0.0.1:8081/slight/hello
  curl -v http://127.0.0.1:8081/wws/hello
  curl -v http://127.0.0.1:8081/lunatic/hello
done

## Install Dapr and init
wget -q https://raw.githubusercontent.com/dapr/cli/master/install/install.sh -O - | /bin/bash -s 1.12.0
dapr uninstall # clean if needed
dapr init -k

## dotnet
dotnet --list-sdks