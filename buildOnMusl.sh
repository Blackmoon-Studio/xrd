#!/bin/sh
export RUSTFLAGS="-Ctarget-feature=-crt-static"
exec cargo b
