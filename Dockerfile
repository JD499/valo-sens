
FROM rust:1.82.0-slim-bookworm as builder


RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY . .


RUN cargo build --release


FROM debian:bookworm-slim


RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app


COPY --from=builder /usr/src/app/target/release/valo-sens /app/valo-sens
COPY assets /app/assets


RUN mkdir -p /app/data


ENV RUST_LOG=info
ENV DATABASE_PATH=/app/data/data.db

EXPOSE 8001

CMD ["./valo-sens"]