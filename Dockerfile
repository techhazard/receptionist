# Build the rust application in a different image
# as we don't need a compiler at runtime
FROM ekidd/rust-musl-builder AS build_image

RUN USER=rust cargo init --name src .
COPY Cargo.* ./
RUN USER=rust nice -n19 cargo build --release

COPY --chown=rust:rust src/ ./src

RUN nice -n19 cargo build --release



# Final Image
FROM nginx:alpine AS final_image

COPY ./html/* /usr/share/nginx/html/

COPY ./start.sh /start.sh
RUN chmod +x /start.sh


COPY --from=build_image /home/rust/src/target/x86_64-unknown-linux-musl/release/receptionist /usr/local/bin/receptionist

ENTRYPOINT ["/start.sh"]
EXPOSE 80
