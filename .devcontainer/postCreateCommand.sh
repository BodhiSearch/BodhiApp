#!/bin/sh
set -e

# https://github.com/devcontainers/features/pull/754/files
sudo chown -R "vscode:vscode" "${RUSTUP_HOME}" "${CARGO_HOME}" && \
    find "${RUSTUP_HOME}" -type d -print0 | xargs -n 1 -0 chmod g+s && \
    find "${CARGO_HOME}" -type d -print0 | xargs -n 1 -0 chmod g+s

# cargo installs
bash /home/vscode/dotfiles/install_devcontainer.sh
export PRE_COMMIT_ALLOW_NO_CONFIG=1
export NEXT_TELEMETRY_DISABLED=1

cd /workspaces/BodhiApp/llamacpp-sys && pre-commit install
cd /workspaces/BodhiApp/llamacpp-sys/llama.cpp && pre-commit install
cd /workspaces/BodhiApp

# Kill the SSH agent
ssh-agent -k
