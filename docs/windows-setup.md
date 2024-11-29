## pwsh
- install pwsh using msi installer

## choco
```pwsh
Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
```

## git

```pwsh
choco install -y git
git config --global core.symlinks true
git config --global core.autocrlf false
git config --global core.eol lf
```

## build tools

```pwsh
choco install -y make
choco install -y cmake
choco install -y llvm
choco install -y nodejs
npm install -g pnpm
```

## python
```pwsh
choco install python3 --version=3.12.2
```

## rust using rustup
- download rustup-init.exe and install default config
`https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe`

- llvm-tools

`rustup component add llvm-tools`

## huggingface cli

```pwsh
python -m pip install -U pip
pip install -U "huggingface_hub[cli]"
```
- download model

`huggingface-cli download TheBloke/Llama-2-7b-Chat-GGUF llama-2-7b-chat.Q4_K_M.gguf`

## checks

```pwsh
cargo build -vv
cargo test -vv

make ci.clean
make ci.coverage
```

