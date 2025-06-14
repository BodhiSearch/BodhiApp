---
title: 'Model Downloads'
description: "Download model files from HuggingFace repositories into Bodhi's local storage"
order: 230
---

# Download Models

Bodhi App allows you to download model files directly from HuggingFace repositories. This feature stores the downloaded model files locally, making them available for use with Bodhi App.

## Overview

When you request to download a model file, you simply provide the HuggingFace repository name and the specific filename of the model (usually a GGUF file). The system then creates a download request and processes it asynchronously.

Key points:

- **Asynchronous Processing:** Download requests are handled in the background. You can monitor the status of your downloads on the Download Models page.
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
   - For any download with an error status, you can expand the row to view detailed error messages.

<p align="center">
  <img 
    src="/doc-images/download-models.jpeg" 
    alt="Download Models Page" 
    class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%]"
  />
</p>

## Error Handling

If a download request cannot be processed, you may see error messages such as:

- **File Already Exists:**  
  The model file already exists in your local storage.
- **Network Error:**  
  An network was not available during the download process.

These errors help you understand the state of your request and take appropriate action, such as checking for duplicate downloads, retrying.

## Next Steps

After submitting a download request:

- Monitor its status on the Downloads page.
- If the status is `completed`, the model file is ready for use.
- If the status is `error`, review the detailed error message to resolve the issue.

---

Happy downloading!

---
