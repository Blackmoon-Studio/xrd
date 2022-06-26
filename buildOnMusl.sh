#!/bin/sh
export RUSTFLAGS="-Ctarget-feature=-crt-static"
cargo clean
mkdir -p target/debug/plugins
cd externs/plugins/xrdCore
cargo b
cp target/debug/libxrdCore.so ../../../target/debug/plugins
cd ../../../
cargo b
exec rm target/debug/libxrdCore.so
