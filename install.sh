#!/usr/bin/bash

cargo build -r
rm ~/.local/bin/ips
cp ./target/release/ips ~/.local/bin/
