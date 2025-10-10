---
title: 'Model Downloads'
description: "Download model files from HuggingFace repositories into Bodhi's local storage"
order: 212
---

# Download Models

Bodhi App allows you to download model files directly from HuggingFace repositories. This feature stores the downloaded model files locally, making them available for use with Bodhi App.

## Overview

When you request to download a model file, you simply provide the HuggingFace repository name and the specific filename of the model (usually a GGUF file). The system then creates a download request and processes it asynchronously.

Key points:

- **Asynchronous Processing:** Download requests are handled in the background. You can monitor the status of your downloads on the Download Models page.
- **Real-Time Progress Tracking:** See download progress with percentage completion and bytes downloaded/total.
- **Background Downloads:** Downloads continue even if you navigate away from the page or close the browser.
- **Idempotency:** If the requested file already exists (based on its repository, filename, and snapshot), the system returns the existing download request rather than creating a duplicate.
- **Error Reporting:** If an error occurs (for example, if the file already exists), the system will notify you with an error message.

## How It Works

1. **Submit a Download Request:**
   - Navigate to the Download Models section in the app.
   - Provide the **repository** (e.g., `TheBloke/Mistral-7B-Instruct-v0.1-GGUF`) and the **filename** (e.g., `mistral-7b-instruct-v0.1.Q8_0.gguf`).
   - The system creates a new download request or returns an existing request if the file is already present.

2. **Processing the Request:**
   - The download request is saved with a status of `pending`.
   - An asynchronous process starts downloading the model file from the specified HuggingFace repository.
   - The status of the download (such as `pending`, `completed`, or `error`) is updated and can be viewed on the Downloads page.

3. **Monitoring Downloads:**
   - On the Downloads page, you can see a table listing all your download requests with details such as repository, filename, status, and timestamp.
   - **Real-Time Progress:** Active downloads show a progress bar with percentage completion and bytes downloaded (e.g., "1.2 GB / 4.5 GB - 27%").
   - **Background Processing:** Downloads continue in the background even if you navigate to other pages or close your browser. The progress is automatically updated when you return to the Downloads page.
   - **Automatic Updates:** The UI automatically polls for progress updates to keep the download status current.
   - For any download with an error status, you can expand the row to view detailed error messages.

<img
  src="/doc-images/download-models.jpeg"
  alt="Download Models Page"
  class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%] mx-auto block"
/>

## Error Handling

If a download request cannot be processed, you may see error messages such as:

- **File Already Exists:**
  The model file already exists in your local storage.
- **Network Error:**
  A network was not available during the download process.

These errors help you understand the state of your request and take appropriate action, such as checking for duplicate downloads, retrying.

## Download Status Tracking

### Status Types

Downloads can have the following statuses:

- **Pending**: Download has been queued and will start shortly
- **In Progress**: Download is actively transferring data (shows progress bar)
- **Completed**: Download finished successfully, model is ready to use
- **Error**: Download failed (expand row for error details)

### Progress Information

For downloads in progress, you'll see:

- **Progress Bar**: Visual representation of completion percentage
- **Bytes Downloaded**: Amount transferred (e.g., "1.2 GB")
- **Total Size**: Complete file size (e.g., "4.5 GB")
- **Percentage**: Completion percentage (e.g., "27%")
- **Download Speed**: Transfer speed provided by HuggingFace library (optimized for maximum performance)
- **Time Remaining**: Estimated time to completion provided by HuggingFace library

### Background Download Behavior

Downloads continue running in the background:

- Navigate freely within Bodhi App while downloads proceed
- Close the browser - downloads continue on the server
- Return anytime to check progress on the Downloads page
- Check the Downloads page to see when downloads complete
- Multiple downloads can run simultaneously

## Next Steps

After submitting a download request:

- Monitor its status on the Downloads page with real-time progress updates
- If the status is `completed`, the model file is ready for use in model aliases
- If the status is `error`, expand the row to review the detailed error message and resolve the issue
- For failed downloads, submit a new download request to retry

---

Happy downloading!

---
