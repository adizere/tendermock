# Builder image
# Because Linux alpine uses `musl-libc` we have to use a special Rust image
FROM ekidd/rust-musl-builder:stable as builder

RUN USER=root cargo new --bin tendermock
WORKDIR ./tendermock

# First build the dependencies (this allow docker to cache them)
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

# Add tendermock sources and compile a release build
ADD ./src ./src
RUN rm ./target/x86_64-unknown-linux-musl/release/deps/tendermock*
RUN cargo build --release

# Final image
FROM alpine:latest
ARG APP=/usr/src/app

# gRPC & JsonRpc
EXPOSE 50051 26657

ENV NODE_USER=tendermock
ENV NODE_GROUP=mock

RUN addgroup $NODE_GROUP \
    && adduser --disabled-password $NODE_USER \
    && addgroup $NODE_USER $NODE_GROUP \
    && mkdir -p ${APP}

COPY --from=builder /home/rust/src/tendermock/target/x86_64-unknown-linux-musl/release/tendermock ${APP}/tendermock

RUN chown -R $NODE_USER:$NODE_GROUP ${APP}

USER $NODE_USER
WORKDIR ${APP}

CMD ["./tendermock"]

