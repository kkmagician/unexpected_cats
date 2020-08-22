FROM centos
RUN mkdir /app
COPY build_target/release/unexpected_cats /app
WORKDIR /app
ENTRYPOINT ["./unexpected_cats"]