---
title: "Installing Bodhi App"
description: "Step-by-step guide to installing Bodhi App on different platforms"
---

# Installing Bodhi App

Bodhi App is available for all major platforms. Follow the instructions for your operating system.

## System Requirements

- **Memory:** 8GB RAM minimum (16GB recommended)
- **Storage:** 10GB free space for base installation
- **Processor:** 64-bit processor
- **Operating System:**
  - Windows 10/11 (64-bit)
  - macOS 11.0 or later
  - Linux (Ubuntu 20.04 or later)
  - Android 9.0 or later
  - iOS 14.0 or later

## Installation Steps

### Windows
1. Download the Windows installer from our releases page
2. Run the `.exe` installer
3. Follow the installation wizard
4. Launch Bodhi App from the Start menu

### macOS
1. Download the macOS `.dmg` file
2. Open the disk image
3. Drag Bodhi App to Applications
4. Launch from Applications folder

### Linux
```bash
# Using apt (Ubuntu/Debian)
curl -fsSL https://bodhi.ai/gpg | sudo gpg --dearmor -o /usr/share/keyrings/bodhi-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/bodhi-archive-keyring.gpg] https://repo.bodhi.ai/apt stable main" | sudo tee /etc/apt/sources.list.d/bodhi.list
sudo apt update
sudo apt install bodhi-app
```

### Mobile Platforms
- **Android:** Install from Google Play Store
- **iOS:** Install from App Store

## Post-Installation

After installation:
1. Launch Bodhi App
2. Choose setup mode (authenticated/non-authenticated)
3. Complete initial configuration 