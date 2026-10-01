#!/bin/sh
# Resolve the service binary from SERVICE_NAME and exec it so the process
# becomes PID 1 and receives signals directly (graceful shutdown).
set -eu

BINARY="${SERVICE_NAME:?SERVICE_NAME must be set at build time}"

if [ ! -x "/app/$BINARY" ]; then
    echo "entrypoint: /app/$BINARY not found or not executable" >&2
    exit 1
fi

exec "/app/$BINARY" "$@"
