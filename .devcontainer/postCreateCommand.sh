#!/bin/sh
set -e

# https://github.com/devcontainers/features/pull/754/files
sudo chown -R "vscode:vscode" "${RUSTUP_HOME}" "${CARGO_HOME}"

# cargo installs
bash /home/vscode/dotfiles/install_devcontainer.sh
export PRE_COMMIT_ALLOW_NO_CONFIG=1

cd /workspaces/BodhiApp
