#!/usr/bin/env bash

set -e

VERSION=$(git rev-parse --short HEAD)

echo $VERSIONdocker push bitcountry/pioneer-collattor-node:

docker build -f scripts/Dockerfile_pioneer . -t bitcountry/pioneer-collattor-node:$VERSION --no-cache --build-arg GIT_COMMIT=${VERSION}
$VERSION
