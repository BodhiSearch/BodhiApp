# Bodhi App

Run Open Source/Open Weight Large Lanuage Models locally.

Bodhi App runs Open Source LLMs locally. It also exposes these LLM inference capabilities as OpenAI API compatible REST APIs. This allows GenAI based native/local applications use the user's GPU/CPU to run inference and provide LLM features without any paid remote API calls.

## llama.cpp and Huggingface Ecosystem

Bodhi App does not re-invent the wheel, and uses [llama.cpp](https://github.com/ggerganov/llama.cpp) to run the Open Source model files of [GGUF format](https://github.com/ggerganov/llama.cpp/tree/master/gguf-py).

It also leverages the rich huggingface.co ecosystem, and uses the existing **$HF_HOME** downloaded models, and current session token to download new model files in a huggingface repo compatible manner. This saves you a lot of local storage and bandwidth by not duplicating the effort.

# Installation

## Homebrew

To install via Homebrew, add `BodhiSearch/homebrew-apps` as an external tap:

```shell
brew tap BodhiSearch/apps
```

Then install the cask bodhi:
```shell
brew install --cask bodhi
```

## Download

Download the latest release for your platform from [Github Release Page](https://github.com/BodhiSearch/BodhiApp/releases).

## Verify Installation

Once the installation is complete, verify the installation:
1. invoke the CLI -
```shell
bodhi -h
```

2. launch `Bodhi.app` from `/Applications` folder, and check the system tray for application icon.

3. Open the homepage using system tray or opening website in the browser - `http://localhost:1135`

# Quick Start

`bodhi run llama3:instruct`

Runs the llama3-8B instructions fine tuned model in a closed-loop (no server) terminal based chat mode. 

The above downloads ~8GB of model files from huggingface.co. If you want a quicker quickstart try:

`bodhi run tinyllama:instruct`

This requires downloading ~0.5GB model. But it is not going to be as powerful as the Llama3 model in its capabilities.

## Text Generation vs Chat Completions

OpenAI has deprecated the Text Generation endpoint, and now only supports Chat Completion endpoints. Following the same trend, Bodhi does not support Text Generation endpoints, and provides Chat Completion endpoint only.

So for chat completion, you need to use a RLHF/Instruct fine-tuned models rather than base model with no intruction fine-tuning. Bodhi requires a `tokenizer_config.json` to convert the User-AI assistant chat into the LLM prompt input to create a model config alias.

## Other Popular Models

| Model Alias    | Parameters | Size    | Quick Start Command                     |
| -------------- | ---------- | ------- | --------------------------------------- |
| llama3 8B      | 8B         | 4.7 GB  | `bodhi run llama3:instruct`             |
| llama3 70B     | 70B        | 40.0 GB | `bodhi run llama3:70b-instruct`         |
| llama2         |            |         | `bodhi run llama2:chat`                 |
| llama2 13B     |            |         | `bodhi run llama2-13b:chat`             |
| llama2 70B     |            |         | `bodhi run llama2-70b:chat`             |
| phi3 mini      |            |         | `bodhi run phi3:mini`                   |
| mistral 7B     |            |         | `bodhi run mistral:instruct`            |
| gemma 7b       |            |         | `bodhi run gemma:instruct`              |
| gemma 7b 1.1   |            |         | `bodhi run gemma:7b-instruct-v1.1-q8_0` |
| tinyllama 1.1B |            |         | `bodhi run tinyllama:instruct`          |


## Import from GGUF

Bodhi supports creating a model config alias using GGUF files from Huggingface. To create a new alias, you need -
1. Huggingface Repo and filename of the GGUF model you want to use
2. Repo hosting `tokenizer_config.json` for the above model
3. Alias name unique to your local setup

`bodhi create <ALIAS> --repo <REPO> --filename <FILE> --tokenizer-config <TOKENIZER_REPO>`

For e.g. to run tinyllama locally, we can create our own custom model using:
1. the model file from huggingface repo [TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF](https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF) using the quantized file [tinyllama-1.1b-chat-v1.0.Q4_0.gguf](https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/blob/main/tinyllama-1.1b-chat-v1.0.Q4_0.gguf)
2. Use the `tokenizer_config.json` from the original tinyllama repository [TinyLlama/TinyLlama-1.1B-Chat-v1.0](https://huggingface.co/TinyLlama/TinyLlama-1.1B-Chat-v1.0)
3. Call it `tinyllama:mymodel`

```shell
bodhi create tinyllama:mymodel \
  --repo TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF \
  --filename tinyllama-1.1b-chat-v1.0.Q4_0.gguf \
  --tokenizer-config TinyLlama/TinyLlama-1.1B-Chat-v1.0
```

Once the alias is created, you can run the above model in interactive mode using:

`bodhi run tinyllama:mymodel`

# Convert Huggingface model to GGUF format

You can convert a Huggingface model to GGUF format using Python library [GGUF](https://pypi.org/project/gguf/).

# CLI

## `bodhi --help`
See the various subcommands supported by the bodhi CLI.

To see the help specific to a subcommand, use:

`bodhi <subcommand> --help`

## `bodhi envs`

Bodhi App can be configured using environment variables. 2 of the important environment variable to configure are:

1. **HF_HOME**
HF_HOME environment variable determines the location of storing huggingface downloaded files, as well as the token to use to query the huggingface endpoint.
By default, it is `$USER_HOME/.cache/huggingface`.

<br/>

2. **BODHI_HOME**
BODHI_HOME environment variable determines the location of storing the config files used by Bodhi App. On the first run, the application sets up BODHI_HOME if not already setup, and creates `aliases` folder to store the aliases yaml configs, `bodhi.sqlite` to store the chat conversations, and `.env` file to load the default environment variables.
By default, it is `$USER_HOME/.cache/bodhi`.

There are other configs set using environment variables. You can list all the current values of environment variables used by the current setup using:

`bodhi envs`

## `bodhi list`

To list the locally configured model aliases:

`bodhi list`

To view the list of pre-configured quickstart model aliases:

`bodhi list --remote`

To view the list of GGUF files in your $HF_HOME:

`bodhi list --models`

## `bodhi pull`

Bodhi allows you to pull any file from huggingface.co given its repo and filename, and store it in **$HF_HOME** in a huggingface repo compatible manner. By default, it pulls the latest version of the file.

`bodhi pull --repo <REPO> --filename <FILENAME>`

## `bodhi create`

We already covered the `bodhi create` as part of [Import from GGUF](#import-from-gguf).


## `bodhi show/edit/cp/rm <ALIAS>`

To view the alias you can use -
`bodhi show <ALIAS>`

To edit the alias in your local editor -
`EDITOR=vi bodhi edit <ALIAS>`

To copy an alias -
`bodhi cp <ALIAS> <NEW-ALIAS>`

To remove the alias -
`bodhi rm <ALIAS>`

## `bodhi serve`

To run a OpenAI compatible API server, run:

`bodhi serve`

This by default starts the server on [http://localhost:1135](http://localhost:1135). You can configure it using command line overrides.

Once the server is started, you query the chat completions endpoint using:

```shell
curl -X POST --location 'http://localhost:1135/v1/chat/completions' \
  --header 'Content-Type: application/json' \
  --data '{
    "model": "tinyllama:instruct",
    "messages": [
        {"role": "user", "content": "List down the days in a calendar week?"}
    ]
  }'
```

# Community

(Open up a pull request on README.md to includ the community integrations)

# Powered By

[llama.cpp](https://github.com/ggerganov/ggml/)
[huggingface.co](https://huggingface.co/)
