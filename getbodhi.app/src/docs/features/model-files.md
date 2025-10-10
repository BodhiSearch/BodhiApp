---
title: 'Model Files'
description: 'View and manage the model files downloaded from HuggingFace.'
order: 215
---

# Model Files

Model Files in Bodhi App provide an overview of the downloaded GGUF models from HuggingFace repositories. This page lists all GGUF model files stored locally along with their repository information, file size, and other metadata. It also offers a direct link to the corresponding HuggingFace repository for each model file.

**Note**: This page displays locally downloaded GGUF models only. For information about configuring API models from providers like OpenAI, Anthropic, and others, see [API Models](/docs/features/api-models).

## Overview

This page displays all the GGUF model files that you have downloaded into your local HuggingFace cache. Bodhi App focuses on GGUF format models for local inference, which are optimized for CPU and GPU execution using llama.cpp.

For each file, you can see details such as:

- **Repository:** The source repository of the model (typically from HuggingFace).
- **Filename:** The name of the GGUF model file (includes quantization level like Q4_K_M, Q8_0).
- **Size:** The storage space used by the model file.
- **Updated At:** The timestamp when the file was last updated.
- **Snapshot:** An identifier for the file version (if available).

An action button is provided for each model file so that you can quickly open the corresponding HuggingFace repository in a new tab.

<img
  src="/doc-images/model-files.jpeg"
  alt="Model Files Page"
  class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%] mx-auto block"
/>

## How It Works

When you navigate to the Model Files page, Bodhi App retrieves and displays all the downloaded model files from your local cache. For each model file, the available action buttons include:

- **Open in HuggingFace:** Clicking this button opens the corresponding repository homepage in your browser.
- **Delete:** Although this feature is coming soon, you will be able to remove model files directly from your local disk.

## Benefits

Using the Model Files page, you are able to:

- **Quickly access** the list of downloaded GGUF models.
- **Monitor storage usage** by viewing the file sizes (GGUF models can range from hundreds of MB to tens of GB).
- **Easily navigate** to the HuggingFace repository to check for updates, model cards, or additional quantization options.
- **Manage your local models** in a central location for a streamlined workflow.
- **Verify downloads** by checking file metadata and snapshot information.

## GGUF Format

Bodhi App uses the GGUF (GPT-Generated Unified Format) for local model inference:

- **Optimized Performance**: GGUF models are optimized for CPU and GPU execution
- **Quantization Support**: Different quantization levels (Q4, Q5, Q8, etc.) balance quality and resource usage
- **Metadata Embedded**: GGUF files contain model metadata for automatic configuration
- **Cross-Platform**: Works on macOS, Windows, and Linux with appropriate hardware acceleration

For information about using non-GGUF models through API providers, see [API Models](/docs/features/api-models).

## Best Practices

- Regularly review the Model Files page to ensure that your local cache is up-to-date.
- Use the link to navigate to the HuggingFace repository for further details about each model.
- Keep an eye out for upcoming features, such as the ability to delete model files directly from the UI.

Happy managing!
