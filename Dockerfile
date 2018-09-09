FROM alpine:latest

VOLUME /data

COPY ./target/x86_64-unknown-linux-musl/release/receptionist /init

RUN ls -lah /
ENTRYPOINT ["/init"]

