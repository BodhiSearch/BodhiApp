/// GptContextParams is now a type alias for Vec<String> to allow direct passthrough
/// of llama-server command-line arguments without structured validation.
pub type GptContextParams = Vec<String>;
