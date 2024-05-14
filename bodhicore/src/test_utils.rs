use rstest::fixture;
use std::{env, fs, path::PathBuf};
use tempfile::TempDir;

use crate::server::BODHI_HOME;

static TEST_REPO: &str = "meta-llama/Meta-Llama-3-8B";
pub static LLAMA2_CHAT_TEMPLATE: &str = r#"{% if messages[0]['role'] == 'system' -%}
  {% set loop_messages = messages[1:] %}{% set system_message = messages[0]['content'] -%}
{% else -%}
  {% set loop_messages = messages %}{% set system_message = false -%}
{% endif -%}
{% for message in loop_messages -%}
  {% if (message['role'] == 'user') != (loop.index0 % 2 == 0) -%}
    {{ raise_exception("Conversation roles must alternate user/assistant/user/assistant/...") }}
  {% endif -%}
  {% if loop.index0 == 0 and system_message != false -%}
    {% set content = '<<SYS>>\\n' + system_message + '\\n<</SYS>>\\n\\n' + message['content'] -%}
  {% else -%}
    {% set content = message['content'] -%}
  {% endif -%}
  {% if message['role'] == 'user' -%}
    {{ bos_token + '[INST] ' + content.strip() + ' [/INST]' -}}
  {% elif message['role'] == 'assistant' -%}
    {{ ' '  + content.strip() + ' ' + eos_token -}}
  {% endif -%}
{% endfor -%}
"#;
pub struct ConfigDirs(pub TempDir, pub PathBuf, pub &'static str);

#[fixture]
pub fn config_dirs(bodhi_home: TempDir) -> ConfigDirs {
  let repo_dir = TEST_REPO.replace('/', "--");
  let repo_dir = format!("configs--{repo_dir}");
  let repo_dir = bodhi_home.path().join(repo_dir);
  fs::create_dir_all(repo_dir.clone()).unwrap();
  ConfigDirs(bodhi_home, repo_dir, TEST_REPO)
}

#[fixture]
pub fn bodhi_home() -> TempDir {
  let bodhi_home = tempfile::Builder::new()
    .prefix("bodhi_home")
    .tempdir()
    .unwrap();
  env::set_var(BODHI_HOME, format!("{}", bodhi_home.path().display()));
  bodhi_home
}
