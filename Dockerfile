FROM rust:latest as builder
COPY . /build
WORKDIR /build
RUN cargo build --release

FROM alpine:latest
COPY build:/build/target/release/unexpected_cats .
ENTRYPOINT ["./unexpected_cats"]