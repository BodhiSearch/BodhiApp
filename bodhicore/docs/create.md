# List Model Alias

```
bodhi list ?<--remote | --models>
```

This command:
- list the model alias configured locally
- `--remote`: list the model pre-configured aliases
- `--models`: list the GGUF model files from local Huggingface cache repo

# Pull a Model
```
bodhi pull \
  <
    alias |
    --repo <repo_id> --file <filename|pattern>
  >
  ?--force
```
This command will:
- `alias`: pull the model using pre-configured alias
  - also sets up the model alias with default `chat-template`, request and model params
- `--repo/--file`: Pull the model file from repo_id/filename|pattern if not exists
- `--force` overwrites the model files if exists

# Create a New Model Alias
```
bodhi create <alias> \
  --repo <repo_id> --file <filename> \
  <
    --chat-template <llama3|llama2|gemma|..> | 
    --tokenizer-config <repo_id|url|local_file>
  >
  --stop-word="" \   # inference parameter
  --temperature="" \ # inference parameter
  --n_ctx="" \       # model load parameter
  --force
```

This command:
- pulls the model from huggingface repo if not exists
- configures to use either built-in chat template using parameter `--chat-template` OR
- pulls the `tokenizer_config.json` from repo provided by parameter `--tokenizer-config`
- configures inference/model load parameters provided
- writes the configuration to `$BODHI_HOME/models/<alias>.yaml`
- `--force`: overwrites the model alias file if exists

# Run a Model using alias
```
bodhi run <alias> \
  <
    --chat-template <llama3|llama2|gemma|...> |
    --tokenizer-config <repo_id|url|local_file>
  >?
  ?--stop-word="" \
  ?--temperature="" \
  ...
```

This command:
- runs the model alias that was created previously
- Optionally, you can overwrite any of the chat-template or model or inference parameters from command line
- the model will run with these overrides
- these overrides will NOT be preserved in a config file

