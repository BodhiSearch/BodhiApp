FROM mcr.microsoft.com/devcontainers/rust:latest-windows

# Install Chocolatey
RUN powershell -Command \
    Set-ExecutionPolicy Bypass -Scope Process -Force; \
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; \
    iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))

# Install Windows dependencies using Chocolatey
RUN choco install -y \
    cmake \
    llvm \
    visualstudio2022buildtools \
    visualstudio2022-workload-vctools \
    git \
    python3 \
    nodejs \
    wget \
    curl \
    vim \
    && choco install -y microsoft-windows-terminal

# Install Rust components and tools
RUN rustup component add llvm-tools-preview rust-src && \
    cargo install cargo-llvm-cov cargo-nextest bindgen-cli tauri-cli@^2.0.0

# Verify installations
RUN rustc --version && cargo --version && rustup --version

# Set Windows-specific environment variables
ENV RUSTUP_HOME="C:\\Users\\vscode\\.rustup" \
    CARGO_HOME="C:\\Users\\vscode\\.cargo"

# Configure Git
RUN git config --system core.autocrlf input
