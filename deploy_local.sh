#!/bin/bash

./build_wasm.sh

cargo install basic-http-server         

basic-http-server deploy/
