## pwsh
- install pwsh using msi installer
https://github.com/PowerShell/PowerShell/releases/download/v7.4.6/PowerShell-7.4.6-win-x64.msi

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
```

## python
```pwsh
choco install python3 --version=3.12.2
```

## visual studio

```
choco install visualstudio2022buildtools `
  --package-parameters "--quiet --wait --norestart --nocache `
  --add Microsoft.VisualStudio.Workload.VCTools `
  --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
  --add Microsoft.VisualStudio.Component.Windows10SDK `
  --includeRecommended" -y --force
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

```pwsh
huggingface-cli download --revision 7ff82c2aaa4dde30121698a973765f39be5288c0 bartowski/microsoft_Phi-4-mini-instruct-GGUF microsoft_Phi-4-mini-instruct-Q4_K_M.gguf
```

## checks

```pwsh
cargo build -vv
cargo test -vv

make ci.clean
make ci.coverage
```

