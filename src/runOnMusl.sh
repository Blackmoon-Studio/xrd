#!/bin/sh
export RUSTFLAGS="-Ctarget-feature=-crt-static"
cargo clean
cd ../externs/plugins/xrdCore
cargo b
cp target/debug/libxrdCore.so ../../../src/plugins
cd ../../../src
exec cargo r
