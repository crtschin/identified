FROM ekidd/rust-musl-builder as planner
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM ekidd/rust-musl-builder as cacher
RUN chown rust:rust .
RUN cargo install cargo-chef
COPY --from=planner home/rust/src/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json


FROM ekidd/rust-musl-builder as builder
RUN chown rust:rust .
COPY . .
COPY --from=cacher home/rust/src/target target
COPY --from=cacher home/rust/.cargo ../.cargo
RUN cargo build --release --bin identified_server


FROM scratch
WORKDIR /usr/src/identified_server
COPY --from=builder /home/rust/target/x86_64-unknown-linux-musl/release/identified_server .
ENTRYPOINT ["./identified_server"]