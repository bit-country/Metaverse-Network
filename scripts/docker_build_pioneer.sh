#!/usr/bin/env bash

set -e

VERSION=$(git rev-parse --short HEAD)

echo $VERSION

docker build -f scripts/Dockerfile_pioneer . -t jerryjren/pioneer-collattor-node:$VERSION --no-cache --build-arg GIT_COMMIT=${VERSION}
docker push jerryjren/pioneer-collattor-node:$VERSION
