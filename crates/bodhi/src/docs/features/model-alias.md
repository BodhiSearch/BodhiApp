---
title: 'Model Aliases'
description: 'Configure and manage your model alias configurations in Bodhi'
order: 210
---

# Model Aliases

Model Aliases in Bodhi App provide a streamlined way to manage and apply LLM configurations for inference.

There are two kinds of model aliases:

1. **User Defined Model Alias**
2. **GGUF Model File Defined Alias**

## User Defined Model Alias

A User Defined Model Alias is essentially a YAML configuration file that contains default request parameters as well as server (context) parameters. This approach makes it easy to reuse and switch between specific model setups without having to reconfigure complex settings each time.

### Sample Model Alias YAML File

All User Defined Model Aliases can be found in the `$BODHI_HOME/aliases` folder. A sample model alias file is shown below:

```yaml
alias: myllama3
repo: QuantFactory/Meta-Llama-3-8B-Instruct-GGUF
filename: Meta-Llama-3-8B-Instruct.Q8_0.gguf
context_params:
  n_ctx: 2048 # Maximum tokens in the prompt
  n_threads: 4 # Number of CPU threads to use
  n_parallel: 1 # Default parallel requests
  n_predict: 4096 # Limit on tokens to generate
  n_keep: 24 # Tokens to keep from the initial prompt
request_params:
  seed: 42 # Ensures determinism
  temperature: 0.7 # Adjusts response randomness
  frequency_penalty: 0.8 # Reduces repetition
  stop:
    - <|start_header_id|>
    - <|end_header_id|>
    - <|eot_id|>
```

A Model Alias YAML file includes the following keys:

- **alias:** (required) A unique name for your model configuration.
- **repo:** (required) The source repository (typically from HuggingFace).
- **filename:** (required) The specific GGUF model file used.
- **snapshot:** (optional) The commit hash if you want to target a specific version; defaults to the `main` branch if omitted.

- **context_params:** Default server settings that impact model initialization:
  - **n_ctx:** Maximum tokens in the prompt.
  - **n_threads:** Number of CPU threads to use.
  - **n_parallel:** Number of requests to process concurrently.
  - **n_predict:** Maximum tokens to generate.
  - **n_keep:** Tokens retained from the initial prompt.
- **request_params:** Default request parameters applied if not specified during a request:
  - **frequency_penalty:** Reduces repetition.
  - **max_tokens:** Limits the length of responses.
  - **presence_penalty:** Encourages topic diversity.
  - **seed:** Ensures reproducible outputs.
  - **stop:** Up to four sequences that, when encountered, halt the response.
  - **temperature:** Adjusts the randomness of responses.
  - **top_p:** Sets the token probability threshold.

## GGUF Model File Defined Alias

A GGUF Model File Defined Alias leverages complete metadata embedded in the GGUF file. In this case, all the default request and context parameters are used, and you cannot override them. This method is the quickest, most direct way to run a model within the app.

The model alias ID for this type is typically a combination of the model repository and the quantization detail. For example, for a repo `QuantFactory/Meta-Llama-3-8B-Instruct-GGUF` and filename `Meta-Llama-3-8B-Instruct.Q8_0.gguf`, the model alias ID would be:

```
QuantFactory/Meta-Llama-3-8B-Instruct-GGUF:Q8_0
```

## How Model Aliases Work

For a **User Defined Model Alias**, when you reference the alias ID in your chat settings or API calls (using the `model` parameter), Bodhi App will:

1. Launch the LLM inference server (if not already running) with the provided `context_params` settings.
2. Apply the `request_params` as the default settings on the request.
3. Forward the request to the inference server.
4. Stream back the response received from the inference server.

Similarly, for a **GGUF Model File Defined Alias**, the process is:

1. Launch the LLM inference server, if not already running, with default server settings.
2. Forward the request to the inference server.
3. Stream back the response.

This approach offers several advantages:

- **Simplicity:** Manage complex configuration details with a single, easy-to-reference alias.
- **Speed:** Quickly start running inferences against a downloaded GGUF file without additional configuration.
- **Consistency:** Ensure that the same parameters are applied across multiple chat sessions or API interactions.
- **Flexibility:** Easily update your configurations via the UI or API, with the server restarting to apply new settings.

## Models Page

The Models page lists all your User Defined Model Aliases as well as GGUF Model File Defined Aliases. From this page, you can browse, edit, and start a chat with any model alias. A copy button appears when you hover over a column value, allowing you to easily copy configuration details.

<p align="center">
  <img 
    src="/doc-images/models-page.jpeg" 
    alt="Models Page" 
    class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%]"
  />
</p>

## Model Alias Form

You can access the New Model Alias form directly from the Models page.

<p align="center">
  <img 
    src="/doc-images/model-alias.jpeg" 
    alt="Model Alias Form" 
    class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%]"
  />
</p>

We hope the above form is self-explanatory.

## Best Practices and Reference Configurations

Bodhi App's Model Alias system is designed to simplify advanced model configuration. By leveraging aliases, you can ensure that each chat session uses a clear, consistent setup tailored to your requirements.

## Performance Considerations

When configuring model aliases, consider these key performance factors:

### Memory Usage vs Thread Count

- Higher thread counts (`n_threads`) can improve inference speed
- But each thread requires additional memory
- Recommended: Start with `n_threads` = number of CPU cores / 2

### Context Size Impact

- Larger context sizes (`n_ctx`) allow for longer conversations
- But increase memory usage and initial load time
- Recommended: Start with 2048 tokens and adjust based on needs

### Quantization Effects

- Lower bit models (Q4_K_M) use less memory but may reduce quality
- Higher bit models (Q8_0) provide better quality but use more memory
- Recommended: Test different quantization levels for your use case

### Optimization Tips

- Set `n_parallel` based on your expected concurrent usage
- Use `n_keep` to maintain important context while reducing memory usage
- Consider using `stop` sequences to prevent unnecessary token generation

Happy configuring!
