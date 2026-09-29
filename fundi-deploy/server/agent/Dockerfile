FROM rust:1.78-bookworm AS build
WORKDIR /app
COPY Cargo.toml ./
RUN mkdir src && echo 'fn main(){}' > src/main.rs && cargo build --release || true
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates wakeonlan arp-scan \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /app/target/release/fundi-deploy /usr/local/bin/fundi-deploy
COPY static ./static
ENV FUNDI_TFTP=/var/lib/tftpboot
ENV FUNDI_IMAGES=/images
EXPOSE 8080
CMD ["fundi-deploy"]
