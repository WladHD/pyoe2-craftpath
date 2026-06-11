# Base image with Flutter and Android SDK pre-installed
# Using specific Flutter version 3.38.9 to match CI workflow
FROM ghcr.io/cirruslabs/flutter:3.38.9

# Switch to root for installing packages
USER root

# uv tool shims land in $HOME/.local/bin - make them available for the root user
ENV PATH="/root/.local/bin:${PATH}"

# Setup sudo for the default user (UID 1000) and create vscode user alias if needed
RUN apt-get update \
    && apt-get install -y sudo \
    && DEFAULT_USER=$(getent passwd 1000 | cut -d: -f1) \
    && if [ -n "$DEFAULT_USER" ]; then \
         echo "$DEFAULT_USER ALL=(root) NOPASSWD:ALL" > /etc/sudoers.d/devuser; \
         if [ "$DEFAULT_USER" != "vscode" ]; then \
           ln -sf /home/$DEFAULT_USER /home/vscode 2>/dev/null || true; \
           useradd -o -u 1000 -g 1000 -d /home/$DEFAULT_USER -s /bin/bash vscode 2>/dev/null || true; \
         fi; \
       else \
         groupadd --gid 1000 vscode; \
         useradd --uid 1000 --gid 1000 -m vscode; \
         echo "vscode ALL=(root) NOPASSWD:ALL" > /etc/sudoers.d/devuser; \
       fi \
    && chmod 0440 /etc/sudoers.d/devuser \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Install system dependencies for Rust backend and additional tools
RUN apt-get update \
    && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    # Rust build dependencies
    clang \
    lld \
    zsh \
    fonts-powerline \
    ttf-ancient-fonts \
    socat \
    protobuf-compiler \
    cmake \
    libssl-dev \
    pkg-config \
    # PostgreSQL client
    postgresql-client \
    # Additional tools
    curl \
    wget \
    git \
    ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH="${CARGO_HOME}/bin:${PATH}"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile default \
    && rustup component add clippy rustfmt \
    && chmod -R a+rw ${RUSTUP_HOME} ${CARGO_HOME}

# Install Node.js (for MCP server)
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Install MCP server
RUN npm install -g @modelcontextprotocol/server-postgres

# Install uv (Python package/tool manager) system-wide
RUN curl -LsSf https://astral.sh/uv/install.sh \
    | env UV_INSTALL_DIR=/usr/local/bin UV_NO_MODIFY_PATH=1 sh

RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/robbyrussell/oh-my-zsh/master/tools/install.sh)"

RUN curl -fsSL https://claude.ai/install.sh | bash

# L3 memory layer: bake the mempalace CLI into the image (uv installed above).
# `uv tool install` brings its own Python and drops the executable in
# /root/.local/bin (already on PATH), so L3 works and persists across rebuilds.
RUN uv tool install mempalace

# Install GitHub CLI (gh) from the official apt repo
RUN mkdir -p -m 755 /etc/apt/keyrings \
    && wget -qO- https://cli.github.com/packages/githubcli-archive-keyring.gpg \
        | tee /etc/apt/keyrings/githubcli-archive-keyring.gpg > /dev/null \
    && chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
        > /etc/apt/sources.list.d/github-cli.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends gh \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Create workspace directory
RUN mkdir -p /workspace \
    && mkdir -p /usr/local/cargo/registry

WORKDIR /workspace

# Stay as root - remoteUser in devcontainer.json handles user switching
