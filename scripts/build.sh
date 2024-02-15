#!/usr/bin/env bash

set -eu

# Build web assets.
(
    cd web_assets
    npm install
    npm run build
)

# Build the binary executable.
(
    cargo build
)
