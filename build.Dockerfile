FROM rust:latest
RUN mkdir /build
WORKDIR /build
ENTRYPOINT ["cargo", "build", "--release"]