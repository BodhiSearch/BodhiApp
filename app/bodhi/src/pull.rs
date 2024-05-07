use crate::{
  hf::{download_async, download_file, download_sync, download_url, model_file},
  hf_tokenizer::{HubTokenizerConfig, TOKENIZER_CONFIG_FILENAME},
  home::configs_dir,
  list::find_remote_model,
};
use anyhow::{anyhow, bail};
use regex::Regex;
use std::path::{Path, PathBuf};
use tokio::runtime::Builder;

#[derive(Debug, PartialEq)]
pub struct Pull {
  pub id: Option<String>,
  pub repo: Option<String>,
  pub file: Option<String>,
  pub config: Option<String>,
  pub force: bool,
}

impl Pull {
  pub fn new(
    id: Option<String>,
    repo: Option<String>,
    file: Option<String>,
    config: Option<String>,
    force: bool,
  ) -> Self {
    Pull {
      id,
      repo,
      file,
      config,
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
    download(repo, file, force)?;
    Ok(())
  }

  fn download_with_id(self) -> anyhow::Result<()> {
    let Pull { id, force, .. } = self;
    let Some(id) = id else {
      bail!("model id is required");
    };
    let model = find_remote_model(&id);
    let Some(model) = model else {
      bail!(
        "model with id {} not found in pre-configured remote models",
        id
      );
    };
    // download(model.repo.clone(), model.default, force)?;
    download_config(
      model.tokenizer_config,
      model.repo.as_str(),
      model.base_model,
    )?;
    Ok(())
  }
}

pub(crate) fn download(repo: String, file: String, force: bool) -> anyhow::Result<PathBuf> {
  let from_cache = model_file(&repo, &file);
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

pub(crate) fn download_config(
  tokenizer_config: String,
  repo: &str,
  base_model: Option<String>,
) -> anyhow::Result<PathBuf> {
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
        &configs_dir(repo)?.join(TOKENIZER_CONFIG_FILENAME),
      )?
    };
    Ok(file_path)
  } else if tokenizer_config.eq("base_model") {
    // config from base_model
    tracing::info!(base_model, repo, "downloading config from base_model");
    match base_model {
      Some(base_model) => {
        let path = download_file(&base_model, TOKENIZER_CONFIG_FILENAME)?;
        Ok(path)
      }
      None => bail!("base_model not found to download config file"),
    }
  } else {
    // config from local path or inline json
    let tokenizer_file = Path::new(&tokenizer_config);
    let config_json = if tokenizer_file.exists() {
      // relative path
      let contents = std::fs::read_to_string(tokenizer_file).map_err(|err| {
        tracing::warn!("failed to read file");
        err
      })?;
      let config_json = serde_json::from_str::<HubTokenizerConfig>(&contents);
      match config_json {
        Ok(config_json) => config_json,
        Err(err) => {
          tracing::warn!("failed to parse file contents as json");
          return Err(err.into());
        }
      }
    } else {
      // inline json
      let config_json = serde_json::from_str::<HubTokenizerConfig>(&tokenizer_config);
      match config_json {
        Ok(config_json) => config_json,
        Err(err) => {
          tracing::warn!("failed to parse inline config as json");
          return Err(err.into());
        }
      }
    };
    let config_dir = configs_dir(repo)?;
    // Serialize config_json as a formatted JSON string
    let config_json_string = serde_json::to_string_pretty(&config_json).map_err(|err| {
      tracing::warn!("failed to serialize config_json");
      err
    })?;

    // Write the JSON string to a file
    let config_file_path = config_dir.join("tokenizer_config.json");
    std::fs::write(&config_file_path, config_json_string).map_err(|err| {
      tracing::warn!("failed to write to file");
      err
    })?;
    Ok(config_file_path)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::{
    hf::{hf_cache, HF_API_PROGRESS, HF_HOME, HF_TOKEN},
    server::BODHI_HOME,
  };
  use hf_hub::Cache;
  use rand::{distributions::Alphanumeric, random};
  use rstest::{fixture, rstest};
  use serial_test::serial;
  use std::{env, fs, io::Write};
  use tempfile::TempDir;

  #[fixture]
  fn tmp_dirs() -> (TempDir, TempDir) {
    env::set_var(HF_API_PROGRESS, "false");
    env::set_var(HF_TOKEN, "");
    let hf_home = tempfile::tempdir().unwrap();
    env::set_var(HF_HOME, hf_home.path().to_string_lossy().into_owned());
    let bodhi_home = tempfile::tempdir().unwrap();
    env::set_var(BODHI_HOME, bodhi_home.path().to_string_lossy().into_owned());
    (hf_home, bodhi_home)
  }

  #[rstest]
  #[serial]
  fn test_download_config_remote_using_hf_url(tmp_dirs: (TempDir, TempDir)) -> anyhow::Result<()> {
    let (hf_home, _) = tmp_dirs;
    let config = download_config(
      "https://huggingface.co/HuggingFaceH4/zephyr-7b-beta/raw/main/tokenizer_config.json"
        .to_string(),
      "HuggingFaceH4/zephyr-7b-beta",
      None,
    )?;
    assert!(config.exists());
    let expected_dest = hf_home
      .path()
      .to_path_buf()
      .join("hub")
      .join("models--HuggingFaceH4--zephyr-7b-beta")
      .join("snapshots")
      .to_string_lossy()
      .into_owned();
    assert!(config.starts_with(expected_dest));
    assert!(config.ends_with(TOKENIZER_CONFIG_FILENAME));
    let tokenizer = HubTokenizerConfig::from_file(&config)
      .ok_or_else(|| anyhow!("error deserializing tokenizer_config.json"))?;
    assert_eq!("<s>", tokenizer.bos_token.unwrap());
    assert_eq!("</s>", tokenizer.eos_token.unwrap());
    Ok(())
  }

  #[rstest]
  #[serial]
  fn test_download_config_remote_using_general_url(
    tmp_dirs: (TempDir, TempDir),
  ) -> anyhow::Result<()> {
    let (hf_home, bodhi_home) = tmp_dirs;
    let config = download_config(
      "https://gist.githubusercontent.com/anagri/5c37dc446cd43a5c751521e117ac4e45/raw/fd459e780f34cf4c6fa07089202919b64aebb658/tokenizer_config.json"
        .to_string(),
      "HuggingFaceH4/zephyr-7b-beta",
      None,
    )?;
    assert!(config.exists());
    let expected_dest = bodhi_home
      .path()
      .to_path_buf()
      .join("configs--HuggingFaceH4--zephyr-7b-beta")
      .join(TOKENIZER_CONFIG_FILENAME);
    assert_eq!(expected_dest, config);
    let tokenizer = HubTokenizerConfig::from_file(&config)
      .ok_or_else(|| anyhow!("error deserializing tokenizer_config.json"))?;
    assert_eq!("<s>", tokenizer.bos_token.unwrap());
    assert_eq!("</s>", tokenizer.eos_token.unwrap());
    Ok(())
  }

  #[rstest]
  #[serial]
  fn test_download_config_using_base_model(tmp_dirs: (TempDir, TempDir)) -> anyhow::Result<()> {
    let (hf_home, bodhi_home) = tmp_dirs;
    let config = download_config(
      "base_model".to_string(),
      "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF",
      Some("TinyLlama/TinyLlama-1.1B-Chat-v1.0".to_string()),
    )?;
    assert!(config.exists());
    let expected_dest = hf_home
      .path()
      .to_path_buf()
      .join("hub")
      .join("models--TinyLlama--TinyLlama-1.1B-Chat-v1.0")
      .join("snapshots")
      .to_string_lossy()
      .into_owned();
    assert!(config.starts_with(expected_dest));
    assert!(config.ends_with(TOKENIZER_CONFIG_FILENAME));
    Ok(())
  }

  #[rstest]
  #[serial]
  fn test_download_config_using_local_file(tmp_dirs: (TempDir, TempDir)) -> anyhow::Result<()> {
    let (hf_home, bodhi_home) = tmp_dirs;
    let tmpdir = tempfile::tempdir()?;
    let mut tmpfile = tmpdir.path().to_path_buf().join("tokenizer_config.json");
    let contents = r#"{"bos_token": "<s>", "eos_token": "</s>"}"#;
    fs::write(&tmpfile, contents)?;

    let config = download_config(
      tmpfile.to_string_lossy().into_owned(),
      "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF",
      None,
    )?;
    assert!(config.exists());
    let expected_dest = bodhi_home
      .path()
      .to_path_buf()
      .join("configs--TheBloke--TinyLlama-1.1B-Chat-v1.0-GGUF")
      .join("tokenizer_config.json")
      .to_string_lossy()
      .into_owned();
    assert_eq!(expected_dest, config.to_string_lossy().into_owned());
    let tokenizer = HubTokenizerConfig::from_file(&config)
      .ok_or_else(|| anyhow!("error deserializing tokenizer_config.json"))?;
    assert_eq!("<s>", tokenizer.bos_token.unwrap());
    assert_eq!("</s>", tokenizer.eos_token.unwrap());
    Ok(())
  }

  #[rstest]
  #[serial]
  fn test_download_config_using_inline_json(tmp_dirs: (TempDir, TempDir)) -> anyhow::Result<()> {
    let (hf_home, bodhi_home) = tmp_dirs;
    let contents = r#"{"bos_token": "<s>", "eos_token": "</s>"}"#.to_string();
    let config = download_config(contents, "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF", None)?;
    assert!(config.exists());
    let expected_dest = bodhi_home
      .path()
      .to_path_buf()
      .join("configs--TheBloke--TinyLlama-1.1B-Chat-v1.0-GGUF")
      .join("tokenizer_config.json")
      .to_string_lossy()
      .into_owned();
    assert_eq!(expected_dest, config.to_string_lossy().into_owned());
    let tokenizer = HubTokenizerConfig::from_file(&config)
      .ok_or_else(|| anyhow!("error deserializing tokenizer_config.json"))?;
    assert_eq!("<s>", tokenizer.bos_token.unwrap());
    assert_eq!("</s>", tokenizer.eos_token.unwrap());
    Ok(())
  }
}
