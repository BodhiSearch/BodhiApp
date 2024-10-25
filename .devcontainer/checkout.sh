#!/bin/sh
set -e

# Ensure SSH agent is running
eval $(ssh-agent -s)
ssh-add ~/.ssh/id_rsa

sudo chown -R vscode:vscode .
# Clone the repository if it doesn't exist
if [ ! -d "/workspaces/BodhiApp/.git" ]; then
  git clone git@github.com:BodhiSearch/BodhiApp.git /workspaces/BodhiApp
  cd /workspaces/BodhiApp
else
  cd /workspaces/BodhiApp
  # Fetch updates but don't apply them yet
  git fetch
  if [ "$(git rev-parse HEAD)" != "$(git rev-parse @{u})" ]; then
    echo "Updates available, pulling changes..."
    git pull
  else
    echo "Repository is already up to date."
  fi
fi

# Update submodules only if there are changes
git submodule update --init --recursive

# Kill the SSH agent
ssh-agent -k

