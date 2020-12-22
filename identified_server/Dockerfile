FROM ekidd/rust-musl-builder as builder

WORKDIR /usr/src/identified

# Avoid having to install/build all dependencies by copying
# the Cargo files and making a dummy src/main.rs
COPY Cargo.toml Cargo.lock ./
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
RUN cargo test
RUN cargo build --release

# We need to touch our real main.rs file or else docker will use the cached one
RUN rm -rf src
COPY ./ .

RUN cargo test
RUN cargo build --release

FROM scratch
WORKDIR /usr/src/identified
COPY --from=builder /home/rust/target//x86_64-unknown-linux-musl/release/identified .

ENTRYPOINT ["./identified"]