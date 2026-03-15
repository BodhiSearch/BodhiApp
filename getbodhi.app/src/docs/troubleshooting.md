---
title: 'Troubleshooting'
description: 'Common issues and solutions for Bodhi App'
order: 600
---

# Troubleshooting

If you encounter issues with Bodhi App, this guide helps you diagnose and resolve common problems. All errors are logged to `$BODHI_HOME/logs` (or stdout before logging initializes).

## Startup Issues

- **Bodhi Home Directory:**
  Bodhi App sets up its home directory at `~/.bodhi`. If this folder is not accessible or writable, the app may crash on startup.
  **Solution**: Verify that `~/.bodhi` exists and has correct read/write permissions.

- **Keychain Access (macOS):**
  On macOS, Bodhi App uses the Keychain to store encryption keys. If Keychain access is denied, the app may fail to start.
  **Solution**: Check Keychain settings and allow Bodhi App access to stored credentials.

- **Port Conflicts:**
  The app defaults to port 1135. If another process is using this port, startup fails.
  **Solution**: Check for processes on port 1135 with `lsof -i :1135` and either stop them or configure a different port.

## Logging

- **Log Files**: Located at `$BODHI_HOME/logs`, rotated daily.
- **Pre-Init Errors**: If the app crashes before logging initializes, errors go to stdout. Run the app from the command line to capture them.
- **Log Levels**: Configure via `BODHI_LOG_LEVEL` environment variable.

## Network & Authentication

- **OAuth Issues:**
  Bodhi App requires an active network connection to refresh tokens from the auth server at `https://id.getbodhi.app/`.
  **Solution**: Ensure network connectivity and that firewall rules permit access to `getbodhi.app`.

- **Model Download Issues:**
  Check your network connection and ensure HuggingFace is accessible.

## MCP Troubleshooting

- **MCP Server Connection Fails:**
  Verify the server URL is correct and the server is running. Check authentication configuration (header auth key, OAuth credentials).

- **Tool Discovery Returns Empty:**
  The MCP server may not expose any tools, or the server may require authentication. Check the server logs and auth configuration.

- **OAuth MCP Authorization Fails:**
  Verify the OAuth configuration (client ID, discovery URL). For DCR servers, ensure the server supports RFC 7591/8414. Try disconnecting and re-authorizing.

- **Playground Execution Errors:**
  Check that the tool is whitelisted in your MCP instance. Non-whitelisted tools show a warning. Verify parameter formats match the tool's input schema.

## Access Request Troubleshooting

### User Access Requests

- **Stuck on Pending:**
  Admin must manually check and approve requests at `/ui/users/`. There are no automatic notifications.

- **Not Redirected After Approval:**
  Your session is invalidated on approval. Log out completely, clear browser cache if needed, and log in again.

- **Request Button Disabled:**
  You may already have a pending request (duplicate prevention), or a submission is in transit. Refresh the page.

### App Access Requests

- **Request Expired:**
  App access request drafts expire after 10 minutes. The third-party app needs to submit a new request.

- **Wrong Resources Approved:**
  Users can only approve resources they have access to. If the app needs additional MCPs, it must submit a new request.

## General Error Diagnostics

1. **Check Logs**: Review `$BODHI_HOME/logs` for error messages corresponding to the time of the issue.
2. **Debug Mode**: Run from the command line to see stdout output.
3. **Docker Logs**: `docker logs <container-name>` for container deployments.

## Issue Resolution Flowchart

1. **Application Launch**
   - Not starting? → Check system requirements, logs at `$BODHI_HOME/logs`, port 1135 conflicts

2. **UI Access**
   - Can't access web interface? → Check network connectivity, firewall, browser console errors

3. **Model Loading**
   - Models not loading? → Verify model file exists, alias configuration is correct, sufficient memory available

4. **API Access**
   - API calls failing? → Check token validity, authorization header format (`Bearer <token>`), scope permissions

5. **MCP Issues**
   - MCPs not working? → Verify server URL, auth config, tool whitelist, check MCP server logs

6. **Performance**
   - Slow inference? → Check model configuration, system resources, ensure correct Docker variant for your GPU

Each step: check `$BODHI_HOME/logs` for detailed error context.

## Getting Help

- **GitHub Issues**: [github.com/BodhiSearch/BodhiApp/issues](https://github.com/BodhiSearch/BodhiApp/issues)
- **Discord**: Community support channel
