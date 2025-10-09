---
title: 'App Settings'
description: 'Configure and manage application settings directly from the UI in Bodhi App'
order: 230
---

# App Settings

Bodhi App provides a built-in configuration management interface that allows administrators to view and update application settings dynamically. Changes take effect immediately without requiring an application restart.

## Overview

The App Settings feature displays all application configuration options, organized into categories. Most settings are **read-only** for informational purposes, showing you the current configuration. A few critical settings are **editable** and can be modified directly through the UI.

### Editable Settings

Currently, you can update the following settings:

- **Execution Variant (BODHI_EXEC_VARIANT):** Select the optimized variant of the Llama.cpp executable that best suits your hardware.
- **Idle Timeout (BODHI_KEEP_ALIVE_SECS):** Set the duration (in seconds) for the application's keep-alive mechanism.

### Read-Only Settings

All other settings are displayed for information and transparency, including:

- Application configuration paths (home directory, data directory, aliases folder)
- Model storage locations (GGUF files in HuggingFace cache, alias YAML files in `$BODHI_HOME/aliases`)
- Server configuration (host, port, bind address)
- Public server settings (public hostname, external access configuration)
- Logging configuration (log level, output format)
- Development information (build version, debug mode)
- Authentication configuration (OAuth URLs, realm settings)
- Runtime configuration (execution variant, timeout settings)

## Understanding Setting Sources

Settings in Bodhi App can come from multiple sources, displayed with color-coded badges for easy identification:

- **System** (Red Badge): Built-in system defaults
- **CLI** (Blue Badge): Command-line arguments passed at startup
- **Env** (Green Badge): Environment variables
- **File** (Orange Badge): Configuration file settings
- **Default** (Gray Badge): Application defaults

### Source Hierarchy

When the same setting is defined in multiple sources, Bodhi App uses a priority hierarchy (highest to lowest):

1. **System** - Highest priority, cannot be overridden
2. **CLI** - Command-line arguments
3. **Env** - Environment variables
4. **File** - Configuration file
5. **Default** - Fallback defaults

The Settings page clearly shows which source is providing the current value, helping you understand your configuration.

## How It Works

1. **Listing Settings:**
   The Settings page displays all application configurations organized into categorized cards. Each setting shows:

   - **Current Value:** The active setting in use
   - **Default Value:** The value that would be applied if not overridden
   - **Source:** Color-coded badge indicating the origin (System/CLI/Env/File/Default)
   - **Edit Button:** Available only for editable settings

2. **Editing Settings:**
   For settings that are editable (currently, execution variant and idle timeout), you can click the **Edit** button to open a dialog where you can modify the value. When you save your changes, the new configuration is sent to the backend, validated, and applied immediately.

3. **Immediate Application:**
   All changes take effect instantly, eliminating the need for any application restart or downtime.

## Using the Settings UI

### Viewing Settings

To view all configuration settings:

- Navigate to the **Settings** page in Bodhi App
- Browse through categorized setting cards
- Check the color-coded source badge to understand where each value comes from
- Compare current values with default values to see what's been customized
- Use the copy functionality to copy setting values for reference

### Updating Editable Settings

To update a configuration:

- Navigate to the **Settings** page in Bodhi App
- Locate the setting you want to change - editable settings have an **Edit** button
- Click **Edit** to open the change dialog
- Modify the value as needed and click **Save**
- The updated setting is applied immediately, and the new value is reflected on the page with an updated source badge

**Note**: Only settings marked as editable can be modified through the UI. Read-only settings can only be changed through environment variables, configuration files, or command-line arguments.

## Supported Settings

Currently, the following settings can be managed via the UI:

- **BODHI_EXEC_VARIANT:** Define the hardware-specific variant for Llama.cpp execution.
- **BODHI_KEEP_ALIVE_SECS:** Adjust the idle timeout duration for the application.

Other configuration options are under development and will be made available in future releases.

<img
  src="/doc-images/app-settings.jpg"
  alt="App Settings Page"
  class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%] mx-auto block"
/>

## Summary

The App Settings feature in Bodhi App empowers administrators to manage system configurations on the fly, offering enhanced flexibility and reducing operational downtime.

Happy configuring!
