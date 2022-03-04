FROM rust:1.58 as builder
WORKDIR /usr/src/myapp
COPY src src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/proxy-service /usr/local/bin/proxy-service
CMD ["proxy-service"]