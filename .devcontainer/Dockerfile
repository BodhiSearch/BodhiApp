FROM mcr.microsoft.com/devcontainers/rust:latest

# Install additional dependencies
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    # build-essential
    build-essential \
    cmake \
    libclang-dev \
    llvm-dev \
    clang \
    # pkg-config
    pkg-config \
    libssl-dev \
    # GTK and basic GUI dependencies
    libgtk-3-dev \
    libgdk-pixbuf2.0-dev \
    libglib2.0-dev \
    # Cairo and Pango for advanced 2D graphics
    libcairo2-dev \
    libpango1.0-dev \
    # ATK for accessibility
    libatk1.0-dev \
    # Soup for HTTP client/server library
    libsoup2.4-dev \
    # WebKit and JavaScriptCore for web content
    libjavascriptcoregtk-4.0-dev \
    libwebkit2gtk-4.0-dev \
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

RUN rustup component add llvm-tools-preview rust-src && \
    cargo install cargo-llvm-cov cargo-nextest bindgen-cli && \
    rustc --version && cargo --version && rustup --version
