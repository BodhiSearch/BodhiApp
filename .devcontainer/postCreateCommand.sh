#!/bin/sh
set -e

# cargo installs
bash /home/vscode/dotfiles/install_devcontainer.sh && pre-commit install
