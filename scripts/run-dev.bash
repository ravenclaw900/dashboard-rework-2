#!/bin/bash -ex

cargo build --bins

./target/debug/server &
sleep 0.1
./target/debug/backend &

trap 'kill $(jobs -pr)' EXIT
wait