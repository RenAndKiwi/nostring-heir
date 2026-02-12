#!/bin/bash
set -e

echo "Building WASM module..."
cd rust
wasm-pack build --target web nostring-heir-ffi

echo "Copying to web app..."
cp nostring-heir-ffi/pkg/*.{js,d.ts} ../web/src/lib/wasm/
cp nostring-heir-ffi/pkg/*.wasm ../web/static/wasm/

echo "Done! WASM module updated."
