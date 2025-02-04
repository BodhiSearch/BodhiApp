---
title: "App Settings"
description: "Configure and manage application settings directly from the UI in Bodhi App"
order: 250
---

# App Settings

Bodhi App provides a built-in configuration management interface that allows administrators to view and update application settings dynamically. Changes take effect immediately without requiring an application restart.

## Overview

The App Settings feature lets you modify various configuration options that determine the behavior and performance of Bodhi App. Currently, you can update the following settings:
- **Execution Variant (BODHI_EXEC_VARIANT):** Select the optimized variant of the Llama.cpp executable that best suits your hardware.
- **Idle Timeout (BODHI_KEEP_ALIVE_SECS):** Set the duration (in seconds) for the application's keep-alive mechanism.

Additional configuration options will be supported in future updates.

## How It Works

1. **Listing Settings:**  
   The Settings page displays a list of application configurations, each with details such as:
   - **Current Value:** The active setting in use.
   - **Default Value:** The value that would be applied if not overridden.
   - **Source:** The origin of the setting (e.g., environment, settings file).

2. **Editing Settings:**  
   For settings that are editable (currently, execution variant and idle timeout), you can click the **Edit** button to open a dialog where you can modify the value. When you save your changes, the new configuration is sent to the backend, validated, and applied immediately.

3. **Immediate Application:**  
   All changes take effect instantly, eliminating the need for any application restart or downtime.

## Using the Settings UI

To update a configuration:
- Navigate to the **Settings** page in Bodhi App.
- Locate the setting you want to change. Editable settings have an **Edit** button.
- Click **Edit** to open the change dialog.
- Modify the value as needed and click **Save**.
- The updated setting is applied immediately, and the new value is reflected on the page.

## Supported Settings

Currently, the following settings can be managed via the UI:
- **BODHI_EXEC_VARIANT:** Define the hardware-specific variant for Llama.cpp execution.
- **BODHI_KEEP_ALIVE_SECS:** Adjust the idle timeout duration for the application.

Other configuration options are under development and will be made available in future releases.

<p align="center">
  <img 
    src="/doc-images/app-settings.jpg" 
    alt="App Settings Page" 
    class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%]"
  />
</p>


## Summary

The App Settings feature in Bodhi App empowers administrators to manage system configurations on the fly, offering enhanced flexibility and reducing operational downtime.

Happy configuring! 