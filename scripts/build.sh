#!/usr/bin/env bash

set -eu

# Set the build mode.
MODE="${MODE:-dev}"

# Build web assets.
(
    cd web_assets
    npm install
    npm run build
)

# Build the binary executable.
(
    if [ "$MODE" = "dev" ]; then
        cargo build
    else
        cargo build --release
    fi
)
