# Llama.cpp Native Support for OAI formats
Supports following params natively:
messages
model
frequency_penalty
logit_bias
logprobs
top_logprobs
max_tokens
presence_penalty
seed
stop
stream
temperature
top_p

# Constraint Values
n: only supports n=1

# Unsupported
tools: unsupported
tool_choice: unsupported

# Extra params required
response_format:
requires "schema" inside response_format to conform to

# Ignores
user
stream_options: ???
