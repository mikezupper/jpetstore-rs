#!/usr/bin/env bash
# Build and push the deployable image.
#
#   ./build-images.sh                # tag = git describe (e.g. deploy-04)
#   ./build-images.sh v1.0.0         # explicit tag
#   REGISTRY=your.registry ./build-images.sh
#
# Two tags every time: the immutable one you can roll back to, and the
# :latest convenience pointer. Assumes `docker login` has been done once.
set -euo pipefail
cd "$(dirname "$0")"

REGISTRY="${REGISTRY:-registry.livepeer.tools}"
IMAGE="$REGISTRY/jpetstore-rs"
VERSION="${1:-$(git describe --tags --always)}"

docker build -t "$IMAGE:$VERSION" -t "$IMAGE:latest" .
docker push "$IMAGE:$VERSION"
docker push "$IMAGE:latest"

echo "pushed $IMAGE:$VERSION and $IMAGE:latest"
