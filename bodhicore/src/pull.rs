use crate::{
  cli,
  hf::{download_async, download_file, download_sync, download_url, model_file},
  hf_tokenizer::{ChatTemplate, HubTokenizerConfig, TOKENIZER_CONFIG_FILENAME},
  home::configs_dir,
  list::find_remote_model,
};
use anyhow::{anyhow, bail};
use regex::Regex;
use std::{fs, path::PathBuf};
use tokio::runtime::Builder;

#[derive(Debug, PartialEq)]
pub struct Pull {
  pub id: Option<String>,
  pub repo: Option<String>,
  pub file: Option<String>,
  pub tokenizer_config: Option<String>,
  pub chat_template: Option<cli::ChatTemplate>,
  pub force: bool,
}

impl Pull {
  pub fn new(
    id: Option<String>,
    repo: Option<String>,
    file: Option<String>,
    tokenizer_config: Option<String>,
    chat_template: Option<cli::ChatTemplate>,
    force: bool,
  ) -> Self {
    Pull {
      id,
      repo,
      file,
      tokenizer_config,
      chat_template,
      force,
    }
  }

  pub fn execute(self) -> anyhow::Result<()> {
    match &self.id {
      Some(_) => {
        self.download_with_id()?;
      }
      None => {
        self.download_with_repo_file()?;
      }
    }
    Ok(())
  }

  fn download_with_repo_file(self) -> anyhow::Result<()> {
    let Pull {
      repo, file, force, ..
    } = self;
    let repo = repo.ok_or_else(|| anyhow!("repo is missing"))?;
    let file = file.ok_or_else(|| anyhow!("file is missing"))?;
    download(&repo, &file, force)?;
    Ok(())
  }

  fn download_with_id(self) -> anyhow::Result<()> {
    let Pull { id, force, .. } = self;
    let Some(id) = id else {
      bail!("unreachable: id should be present");
    };
    let model = find_remote_model(&id).ok_or_else(|| {
      anyhow!(
        "id '{}' not found in pre-configured models.\nCheck pre-configured models using `bodhi list -r`.",
        id
      )
    })?;
    download(&model.repo, &model.default_variant, force)?;
    let tokenizer_config = build_config(
      model.tokenizer_config,
      model.repo.as_str(),
      model.base_model,
    )?;
    // let config_path = configs_dir(&model.repo)?.join("default.yaml");
    // tracing::debug!(?config_path, "saving config to file");
    // let config_file = std::fs::File::create(config_path)?;
    // serde_yaml::to_writer(config_file, &tokenizer_config)?;
    Ok(())
  }
}

pub(crate) fn download(repo: &str, file: &str, force: bool) -> anyhow::Result<PathBuf> {
  let from_cache = model_file(repo, file);
  if let Some(file) = from_cache {
    if !force {
      println!("model file already exists in cache: '{}'", file.display());
      println!("use '--force' to force download it again");
      bail!("");
    }
  }
  let runtime = Builder::new_multi_thread().enable_all().build();
  let path = match runtime {
    Ok(runtime) => runtime.block_on(async move { download_async(repo, file).await })?,
    Err(_) => download_sync(repo, file)?,
  };
  Ok(path)
}

pub(crate) fn build_config(
  tokenizer_config: String,
  repo: &str,
  base_model: Option<String>,
) -> anyhow::Result<HubTokenizerConfig> {
  let tempdir = tempfile::tempdir()?;
  if tokenizer_config.starts_with("https://") {
    // remote
    let hf_file_regex = Regex::new(
      r"^https://huggingface.co/(?P<owner>[^/]+)/(?P<repo>[^/]+)/raw/main/(?P<filename>[^/]+)$",
    )?;
    let file_path = if let Some(captures) = hf_file_regex.captures(&tokenizer_config) {
      let owner = captures.name("owner").unwrap().as_str();
      let repo = captures.name("repo").unwrap().as_str();
      let filename = captures.name("filename").unwrap().as_str();
      download_file(&format!("{}/{}", owner, repo), filename)?
    } else {
      download_url(
        &tokenizer_config,
        &tempdir.path().join("tokenizer_config.json"),
      )?
    };
    let hub_tokenizer_config = HubTokenizerConfig::from_json_file(file_path)?;
    Ok(hub_tokenizer_config)
  } else if tokenizer_config.eq("base_model") {
    // config from base_model
    tracing::info!(base_model, repo, "downloading config from base_model");
    match base_model {
      Some(base_model) => {
        let file_path = download_file(&base_model, TOKENIZER_CONFIG_FILENAME)?;
        let hub_tokenizer_config = HubTokenizerConfig::from_json_file(file_path)?;
        Ok(hub_tokenizer_config)
      }
      None => bail!("base_model not found to download config file"),
    }
  } else {
    unimplemented!()
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::hf::{HF_API_PROGRESS, HF_TOKEN};
  use rstest::{fixture, rstest};
  use serial_test::serial;
  use std::env;

  #[fixture]
  fn setup() {
    env::set_var(HF_API_PROGRESS, "false");
    env::set_var(HF_TOKEN, "");
  }

  #[rstest]
  #[serial]
  fn test_download_config_using_repo(_setup: ()) -> anyhow::Result<()> {
    let config = build_config(
      "base_model".to_string(),
      "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF",
      Some("TinyLlama/TinyLlama-1.1B-Chat-v1.0".to_string()),
    )?;
    assert_eq!("<s>", config.bos_token.unwrap());
    assert_eq!("</s>", config.eos_token.unwrap());
    assert!(!config.chat_template.unwrap().is_empty());
    Ok(())
  }
}
