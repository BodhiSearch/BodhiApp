/* eslint-disable */
/* prettier-ignore-start */

export const SETTINGS_TOOLTIPS = {
  alias: "Choose an AI model or saved custom configuration. Create and manage custom configurations in the Models page.",
  stream: "Show AI responses in real-time as they're generated, instead of waiting for the complete response.",
  apiToken: "Your personal authentication token for API access. Keep secure.",
  seed: "A number that helps generate consistent responses across multiple runs with the same settings.",
  systemPrompt: "Initial instructions that set the AI's role and behavior for the conversation.",
  stopWords: "Up to 4 phrases that will stop the AI's response when encountered.",
  temperature: "Controls creativity: Low (0.2) for focused responses, High (0.8) for more varied outputs.",
  topP: "Alternative to Temperature. Controls response variety by limiting token probability threshold.",
  maxTokens: "Maximum length of AI's response. Higher values allow longer responses.",
  presencePenalty: "Positive values encourage the AI to discuss new topics rather than staying on current ones.",
  frequencyPenalty: "Positive values reduce repetition of words and phrases in responses.",
} as const;
/* prettier-ignore-end */
/* eslint-enable */
