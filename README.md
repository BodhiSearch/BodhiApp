# Bodhi App

Run LLMs locally.

Bodhi App runs LLMs and other GenerativeAI inference locally. It also exposes these features as authenticated services.

This allows GenAI based native applications, chrome extensions, or web pages use the local user's GPU/CPU to run inference and provide GenAI features without any paid API calls.

# Installation

## Homebrew

To install via Homebrew, add `BodhiSearch/homebrew-apps` as an external tap -

```shell
brew tap BodhiSearch/apps
```

Then install the cask bodhi -
```shell
brew install --cask bodhi
```

Once the installation is complete, verify the installation -
1. invoke the CLI -
```shell
bodhi -h
```

2. launch `Bodhi.app` from `/Applications` folder, and check the system tray for application icon.

3. Open the homepage using system tray or opening website in the browser - `http://localhost:1135`
