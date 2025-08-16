#!/bin/bash

# Set up library path for KunRuntime
export LD_LIBRARY_PATH="$PWD/kunquant-env/lib/python3.12/site-packages/KunQuant/runner:$LD_LIBRARY_PATH"

echo "Running KunQuant-rs tests..."
echo "LD_LIBRARY_PATH: $LD_LIBRARY_PATH"

# Run all tests
cargo test -- --nocapture
