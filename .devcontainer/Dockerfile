FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

# Install dependencies for Windows cross-compilation
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    mingw-w64 \
    pkg-config \
    build-essential \
    curl \
    git \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN rustup update stable
RUN rustup default stable-x86_64-unknown-linux-gnu
RUN rustup component add rustfmt

# Add Windows targets for Rust
RUN rustup target add \
    x86_64-pc-windows-gnu

# Optional: verify setup
RUN rustup show && \
    rustup target list --installed && \
    x86_64-w64-mingw32-gcc --version
