#!/bin/bash
set -e

echo "Building StreamK8s Plugin (WASM)..."
cargo build --target wasm32-unknown-unknown --release

# Note: In a real flow, we would copy the wasm file to a bundle folder.
echo "Build complete! Artifact is in target/wasm32-unknown-unknown/release/rust_plugin_template.wasm"
