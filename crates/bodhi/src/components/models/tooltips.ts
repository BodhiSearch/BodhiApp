/* eslint-disable */
/* prettier-ignore-start */

export const ALIAS_FORM_TOOLTIPS = {
  alias: "A unique name for your model configuration. Use this to reference your settings in chat.",
  repo: "The repository containing your model file. Usually points to a Hugging Face repository.",
  filename: "The GGUF model file to use from the selected repository.",
  chatTemplate: "Template that defines how to format messages for the model.",

  // Request Parameters
  temperature: "Controls creativity: Low (0.2) for focused responses, High (0.8) for more varied outputs.",
  top_p: "Alternative to Temperature. Controls response variety by limiting token probability threshold.",
  max_tokens: "Maximum length of AI's response. Higher values allow longer responses.",
  presence_penalty: "Positive values encourage the AI to discuss new topics rather than staying on current ones.",
  frequency_penalty: "Positive values reduce repetition of words and phrases in responses.",
  stop: "Up to 4 phrases that will stop the AI's response when encountered.",
  seed: "A number that helps generate consistent responses across multiple runs with the same settings.",

  // Context Parameters
  n_ctx: "Size of the prompt context. Affects memory usage and response context.",
  n_seed: "A number that helps generate consistent responses across multiple runs with the same settings.",
  n_threads: "Number of CPU threads to use. Affects processing speed.",
  n_parallel: "Number of concurrent requests to process. Impacts server load.",
  n_predict: "Maximum number of tokens to generate. Controls response length.",
  n_keep: "Number of tokens to keep from the initial prompt. Affects context preservation.",
} as const;

/* prettier-ignore-end */
/* eslint-enable */
