# Build the pyoe2-backend binary (REST / worker / MCP / CLI in one image).
# Build context must be the repository root (the proto/ dir is shared):
#   docker build -f backend/Dockerfile .

# bookworm pin keeps the builder's glibc compatible with distroless cc-debian12
FROM rust:1.96-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config make perl gcc g++ ca-certificates protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY proto ./proto
COPY backend ./backend

WORKDIR /build/backend
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/backend/target \
    cargo build --release --locked -p craftpath-server \
    && cp target/release/pyoe2-backend /usr/local/bin/pyoe2-backend

# ---------------------------------------------------------------------------

FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/local/bin/pyoe2-backend /usr/local/bin/pyoe2-backend
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

ENV CACHE_DIR=/cache
EXPOSE 8080 8081

ENTRYPOINT ["/usr/local/bin/pyoe2-backend"]
CMD ["rest"]
