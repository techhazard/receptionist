#!/bin/bash -e

docker build . -t rust -f Dockerfile-rust
docker run -ti -v "$(pwd):/app" rust --release --target=x86_64-unknown-linux-musl

docker build . -t app
