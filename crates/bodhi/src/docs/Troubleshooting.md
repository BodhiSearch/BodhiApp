---
title: 'Troubleshooting'
description: 'Common issues and solutions'
order: 600
---

# Troubleshooting

If you encounter issues with Bodhi App, this guide can help you diagnose and resolve common problems. All errors are logged (either to the log files or stdout) to help trace the source of the issue.

## Startup Issues

- **Bodhi Home Directory:**  
  Bodhi App sets up its home directory inside `$HOME/.cache/bodhi`. If access to this folder is denied or if it isn't writable, the app may crash during startup.  
  **Solution:**

  - Verify that the `$HOME/.cache/bodhi` folder exists and has the correct permissions.
  - Ensure you have read/write access to this directory.

- **Keychain Access (macOS):**  
  On macOS, Bodhi App uses the Keychain to store encryption keys for sensitive data. If access to the Keychain is denied or if an error occurs, the app may fail to start.  
  **Solution:**
  - Check your system Keychain settings and allow Bodhi App to access necessary credentials.

Any startup error details are logged—see the **Logs** section below for further diagnostics.

## Logging Configuration

- **Log Files:**  
  Once initialized, Bodhi App writes logs to `$BODHI_HOME/logs` and rotates them daily. In the event the app crashes before logging is fully set up, error messages may be output to STDOUT.  
  **Solution:**

  - If you suspect logging-related issues, run Bodhi App from the command line to capture output in real-time.
  - Ensure the logs directory exists and has proper permissions.

- **STDOUT Logging:**  
  When the app is starting up, and logging isn't initialized, errors are sent to the system STDOUT. Running the app via command line is recommended to capture such logs.

Bodhi App has a robust error handling mechanism that captures issues and origin (for example, misconfigured environment variables). These errors are logged with detailed context.

**Solution:**

- Check the log files for error messages related to feature settings.
- Verify that any required environment variables are set correctly.
- Confirm that the default settings are properly defined in your configuration file.

## Network & External Dependencies

- **Authentication and Remote Dependencies:**  
  Bodhi App relies on an active network connection to interact with remote services—for example, to refresh access tokens from the authentication server at `https://id.getbodhi.app/`.
  **Solution:**
  - Ensure your network connection is active.
  - Confirm that firewall rules permit access to external domains such as `getbodhi.app`.
- **Model Download Issues:**  
  If model downloads are failing, check your network connection and ensure that the Huggingface repository can be accessed.

## General Error Diagnostics

- **Locating Logs:**  
  Detailed error messages are logged in `$BODHI_HOME/logs`. Review these files to diagnose issues. The logs rotate daily, so check the logs corresponding to the time of the error.
- **Debug Mode:**  
  If issues persist, launch Bodhi App from the command line in debug mode. This will output error messages to STDOUT, offering immediate insight into what might be going wrong.

If you continue to experience issues after following these steps, please reach out via our Discord channel or submit an issue on Github.

## Issue Resolution Flowchart

Follow this decision tree to diagnose common issues:

1. **Application Launch**

   - Is the app starting?
     - No → Check:
       - System requirements
       - Log files in `$BODHI_HOME/logs`
       - Process conflicts on port 1135
     - Yes → Proceed to step 2

2. **UI Access**

   - Can you access the web interface?
     - No → Check:
       - Network connectivity
       - Firewall settings
       - Browser console for errors
     - Yes → Proceed to step 3

3. **Model Loading**

   - Are models loading correctly?
     - No → Verify:
       - Model file exists
       - Model alias configuration
       - Available system memory
     - Yes → Proceed to step 4

4. **API Access**

   - Are API calls working?
     - No → Check:
       - API token validity
       - Authentication mode
       - Request format
     - Yes → Proceed to step 5

5. **Performance Issues**
   - Is performance satisfactory?
     - No → Review:
       - Model configuration
       - System resources
       - Network latency
     - Yes → System is working as expected

Each step includes detailed logs in `$BODHI_HOME/logs` to help identify specific issues.
