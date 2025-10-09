---
title: "Chat UI"
description: "A comprehensive guide to using Bodhi App's Chat Interface"
order: 201
---

# Chat UI

Welcome to Bodhi App's Chat UI! This guide is designed to help you get started with our conversational AI interface. Whether you are a first-time user or someone looking to explore advanced configuration options, you will find all the information you need in this guide.

## Overview

Bodhi App's Chat UI features a clean, three-panel design that keeps everything you need at your fingertips. The interface is divided into:

- **Chat History Panel (Left):** View and manage your past conversations.
- **Main Chat Panel (Center):** Interact directly with the AI assistant.
- **Settings Panel (Right):** Configure the AI's behavior using various parameters.

<img 
  src="/doc-images/chat-ui.jpeg" 
  alt="Chat UI" 
  class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%]"
/>

Every conversation and setting is stored locally in your browser. This means your data is private but will be lost if you clear your browser data.

## The Chat History Panel

The left panel displays your previous conversations grouped by the time they were started—such as *Today*, *Yesterday*, and *Previous 7 Days*. You can click on any conversation to reopen it. A dedicated delete option lets you permanently remove a conversation from your browser's local storage, so use it with caution.

## The Main Chat Panel

The center panel is where the conversation happens. Here you can:

- Type your message in the input field at the bottom.
- Press **Enter** (or click the **+** icon) to start a new chat or submit your message.
- Enjoy real-time streaming of AI responses, or see complete responses once processing is finished.
- Experience rich content rendering: Markdown is converted to HTML, and code blocks are syntax highlighted
- Copy the response or code block using Copy button

## The Settings Panel

The right panel is your command center for configuring how the AI responds. Here's what you need to know:

- **Model/Alias Selection:**  
  Choose from available GGUF models or your custom aliases using a dropdown selector.
  
- **Adjustable Parameters:**  
  Customize AI behavior with controls that include sliders, numeric inputs, and tag-based inputs:
  
  - **Temperature:** Adjusts response creativity.
  - **Top P:** Sets the probability threshold for token selection.
  - **Seed:** Ensures consistency in generated responses.
  - **Max Tokens:** Determines the maximum response length.
  - **Stop Words:** Allows specification of up to four phrases to stop the response immediately.
  
  Each setting is paired with a tooltip to help you understand its impact. In addition, every control comes with a toggle switch. When disabled, the system reverts to backend default values.
  
- **Security Notice:**  
  If you use your API token, remember it is stored in plain text on your browser. For security, remove it from the input after you're done.
  
- **Popular Reference Parameters:**  
  For advanced users, here are sample configurations:
  
  - **Creative Writing:**  

```yaml
Temperature: 0.8  
Top P: 0.9  
Presence Penalty: 0.6
Frequency Penalty: 0.3
Max Tokens: 2048
```  
  - **Technical Responses:**  

```yaml
Temperature: 0.2
Top P: 1.0
Presence Penalty: 0.1
Frequency Penalty: 0.1
```

  - **Balanced Conversation:**  

```yaml
Temperature: 0.5
Top P: 1.0
Presence Penalty: 0.4
Frequency Penalty: 0.4
```

## Collapsible Panels & Starting a New Chat

Both the Chat History and Settings Panels are collapsible. This allows you to maximize your workspace if you have limited screen space or wish to focus solely on your conversation. You can toggle each panel independently.

You have two options to start a new conversation:
- Click the **+** button in the main chat input area.
- Use the new chat option in the Chat History Panel.

## Final Thoughts

Bodhi App's Chat UI is thoughtfully designed to combine ease of use with powerful functionality. Enjoy interacting with the AI assistant and experiment with the settings to tailor the experience to your needs. Always remember that your configurations and history are stored locally, so manage your data wisely.

Happy chatting! 