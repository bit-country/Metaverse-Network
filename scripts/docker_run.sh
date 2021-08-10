#!/usr/bin/env bash

set -e

VERSION=$(git rev-parse --short HEAD)

echo $VERSION

docker build -f scripts/Dockerfile_dev . -t bitcountry/bitcountry-node:$VERSION --no-cache --build-arg GIT_COMMIT=${VERSION}
docker push bitcountry/bitcountry-node:$VERSION
