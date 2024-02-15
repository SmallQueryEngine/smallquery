#!/usr/bin/env bash

set -eu

# Build the binary executable.
(
    MODE=release scripts/build.sh
)

# Run the binary executable.
(
    cargo run --release
)