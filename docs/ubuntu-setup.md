## python
```bash
sudo apt install python3.10 python-is-python3 python3-pip
export PATH=$PATH:$HOME/.local/bin
```

## git config

```bash
git config --global core.symlinks true
git config --global core.autocrlf false
git config --global core.eol lf
```

## build tools

```bash
sudo apt install -y \
    --no-install-recommends \
    build-essential \
    cmake \
    libclang-dev \
    llvm-dev \
    clang \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libgdk-pixbuf2.0-dev \
    libglib2.0-dev \
    libcairo2-dev \
    libpango1.0-dev \
    libatk1.0-dev \
    libsoup2.4-dev \
    libjavascriptcoregtk-4.0-dev \
    libwebkit2gtk-4.1-dev \
    curl \
    wget \
    file \
    jq \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    vim \
    tig \
    pre-commit && \
    sudo apt-get autoremove -y && \
    sudo apt-get clean -y && \
    sudo rm -rf /var/lib/apt/lists/*
```

## rust using rustup
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup component add llvm-tools rust-src
cargo install cargo-llvm-cov tauri-cli@^2.0.0
```

## nodejs
```bash
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
```

## huggingface-cli
```bash
python -m pip install -U pip
python -m pip install -U "huggingface_hub[cli]"
```

## download the model
```bash
huggingface-cli download --revision 7ff82c2aaa4dde30121698a973765f39be5288c0 bartowski/microsoft_Phi-4-mini-instruct-GGUF microsoft_Phi-4-mini-instruct-Q4_K_M.gguf
```

## checks

```bash
cargo build -vv
cargo test -vv

make ci.clean
make ci.coverage
```

