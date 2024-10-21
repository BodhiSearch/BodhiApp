#!/bin/sh
set -e

echo "Installing GTK and related dependencies..."

apt-get update
export DEBIAN_FRONTEND=noninteractive

# GTK and basic GUI dependencies
apt-get -y install --no-install-recommends \
    libgtk-3-dev \
    libgdk-pixbuf2.0-dev \
    libglib2.0-dev

# Cairo and Pango for advanced 2D graphics
apt-get -y install --no-install-recommends \
    libcairo2-dev \
    libpango1.0-dev

# ATK for accessibility
apt-get -y install --no-install-recommends \
    libatk1.0-dev

# Soup for HTTP client/server library
apt-get -y install --no-install-recommends \
    libsoup2.4-dev

# WebKit and JavaScriptCore for web content
apt-get -y install --no-install-recommends \
    libjavascriptcoregtk-4.0-dev \
    libwebkit2gtk-4.0-dev

# Clean up
apt-get autoremove -y
apt-get clean -y
rm -rf /var/lib/apt/lists/*

# Set PKG_CONFIG_PATH
echo 'export PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig:$PKG_CONFIG_PATH"' >> /etc/zsh/zshenv
echo 'export PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig:$PKG_CONFIG_PATH"' >> /etc/bash.bashrc

echo "GTK and related dependencies installed successfully."

# cargo installs
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov cargo-nextest
