#!/bin/bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown

rm -r deploy/
mkdir deploy

cp -r static/ deploy
cp -r resources/ deploy/resources
cp target/wasm32-unknown-unknown/release/transport-io.wasm deploy

