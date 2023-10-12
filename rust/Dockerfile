FROM rust:alpine as builder

RUN apk add --no-cache pkgconfig openssl-dev libc-dev

WORKDIR /usr/src/bench

COPY Cargo.toml Cargo.lock ./
RUN mkdir src
COPY src ./src

# https://github.com/rust-lang/cargo/issues/10781#issuecomment-1163819998
RUN rustup update nightly
RUN cargo +nightly -Z sparse-registry update
RUN cargo +nightly build --release -Z sparse-registry

FROM alpine:latest

COPY --from=builder /usr/src/bench/target/release/bench /usr/local/bin/bench

CMD ["bench"]
