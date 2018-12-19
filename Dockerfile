# Build the rust application in a different image
# as we don't need a compiler at runtime
FROM ekidd/rust-musl-builder AS build_image

# cargo requires USER to be set
USER rust
ARG USER=rust

# we start with a dummy crate to just install the dependencies
RUN cargo init --bin --name src .

# copy just our project's Cargo.toml and build the dependencies
COPY Cargo.* ./

# We run with `nice -n19` to not interrupt
# anything else going on on the system.
#
# Afterwards, we set the timestamp to
# 1970 to make sure all changes require
# a rebuild. If we start a docker build
# without cache, the source of the dummy
# crate would have been newer and would
# not trigger a rebuild.
RUN nice -n19 cargo build --release \
 && find target src | xargs touch -t 197001011200

# Build the project with built dependencies
COPY src/ ./src
RUN nice -n19 cargo build --release



# Final Image
FROM nginx:alpine AS final_image

# Copy the static css and js files
COPY ./html/* /usr/share/nginx/html/

COPY ./start.sh /start.sh
RUN chmod +x /start.sh


COPY --from=build_image /home/rust/src/target/x86_64-unknown-linux-musl/release/receptionist /usr/local/bin/receptionist

ENTRYPOINT ["/start.sh"]
EXPOSE 80
