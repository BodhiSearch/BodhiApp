FROM --platform=linux/amd64 mcr.microsoft.com/devcontainers/rust:latest

# Install additional dependencies
RUN dpkg --add-architecture amd64 && \
    apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    # build-essential
    build-essential:amd64 \
    cmake:amd64 \
    libclang-dev:amd64 \
    llvm-dev:amd64 \
    clang:amd64 \
    # pkg-config
    pkg-config:amd64 \
    libssl-dev:amd64 \
    # GTK and basic GUI dependencies
    libgtk-3-dev:amd64 \
    libgdk-pixbuf2.0-dev:amd64 \
    libglib2.0-dev:amd64 \
    # Cairo and Pango for advanced 2D graphics
    libcairo2-dev:amd64 \
    libpango1.0-dev:amd64 \
    # ATK for accessibility
    libatk1.0-dev:amd64 \
    # Soup for HTTP client/server library
    libsoup2.4-dev:amd64 \
    # WebKit and JavaScriptCore for web content
    libjavascriptcoregtk-4.0-dev:amd64 \
    libwebkit2gtk-4.1-dev:amd64 \
    # tauri v2
    curl \
    wget \
    file \
    libxdo-dev:amd64 \
    libssl-dev:amd64 \
    libayatana-appindicator3-dev:amd64 \
    librsvg2-dev:amd64 \
    # dev tools
    vim \
    tig \
    pre-commit \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/* \
    # Set PKG_CONFIG_PATH
    && echo 'export PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig:$PKG_CONFIG_PATH"' >> /etc/zsh/zshenv \
    && echo 'export PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig:$PKG_CONFIG_PATH"' >> /etc/bash.bashrc

# Clean up
RUN apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Set platform-specific environment variables
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc

RUN rustup component add llvm-tools-preview rust-src && \
    cargo install cargo-llvm-cov cargo-nextest bindgen-cli tauri-cli@^2.0.0 && \
    rustc --version && cargo --version && rustup --version
