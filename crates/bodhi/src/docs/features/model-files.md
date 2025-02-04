---
title: "Model Files"
description: "View and manage the model files downloaded from HuggingFace."
order: 220
---

# Model Files

Model Files in Bodhi App provide an overview of the downloaded models from HuggingFace repositories. This page lists all model files stored locally along with their repository information, file size, and other metadata. It also offers a direct link to the corresponding HuggingFace repository for each model file.

## Overview

This page displays all the model files that you have downloaded into your local HuggingFace cache. For each file, you can see details such as:

- **Repository:** The source repository of the model.
- **Filename:** The name of the GGUF model file.
- **Size:** The storage space used by the model file.
- **Updated At:** The timestamp when the file was last updated.
- **Snapshot:** An identifier for the file version (if available).

An action button is provided for each model file so that you can quickly open the corresponding HuggingFace repository in a new tab.

<p align="center">
  <img 
    src="/doc-images/model-files.jpeg" 
    alt="Model Files Page" 
    class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%]"
  />
</p>

## How It Works

When you navigate to the Model Files page, Bodhi App retrieves and displays all the downloaded model files from your local cache. For each model file, the available action buttons include:

- **Open in HuggingFace:** Clicking this button opens the corresponding repository homepage in your browser.
- **Delete:** Although this feature is coming soon, you will be able to remove model files directly from your local disk.

## Benefits

Using the Model Files page, you are able to:

- **Quickly access** the list of downloaded models.
- **Monitor storage usage** by viewing the file sizes.
- **Easily navigate** to the HuggingFace repository to check for updates or additional information.
- **Manage your models** in a central location for a streamlined workflow.

## Best Practices

- Regularly review the Model Files page to ensure that your local cache is up-to-date.
- Use the link to navigate to the HuggingFace repository for further details about each model.
- Keep an eye out for upcoming features, such as the ability to delete model files directly from the UI.

Happy managing!
