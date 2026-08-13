#!/usr/bin/env bash
# Deploy a version — and roll one back. Same verb, different tag:
#
#   ./redeploy.sh                # latest
#   TAG=deploy-08 ./redeploy.sh  # any immutable tag, older or newer
#
# compose only recreates what changed; the data volume is external and
# never in play. Rollback isn't an emergency procedure, it's a redeploy
# with a smaller number.
set -euo pipefail
cd "$(dirname "$0")"

export TAG="${TAG:-latest}"
docker compose pull petstore
docker compose up -d
docker compose ps petstore
