#!/usr/bin/env bash

set -eu

# Build the binary executable.
(
    scripts/build.sh
)

# Run the binary executable.
(
    cargo run
)