# PACKAGE.md

See [CLAUDE.md](./CLAUDE.md) for architectural guidance and design rationale.

## Core Components

### Server Trait and Implementation

**Server Trait** (`crates/llama_server_proc/src/server.rs:83-101`)
```rust
#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait Server: std::fmt::Debug + Send + Sync {
  async fn start(&self) -> Result<()>;
  async fn stop(self: Box<Self>) -> Result<()>;
  async fn chat_completions(&self, body: &Value) -> Result<Response>;
  async fn embeddings(&self, body: &Value) -> Result<Response>;
  // ...
}
```

**LlamaServer Implementation** (`crates/llama_server_proc/src/server.rs:103-133`)
```rust
#[derive(Debug)]
pub struct LlamaServer {
  process: Mutex<Option<Child>>,
  client: reqwest::Client,
  base_url: String,
  executable_path: PathBuf,
  server_args: LlamaServerArgs,
}
```

### Configuration Management

**LlamaServerArgs Builder** (`crates/llama_server_proc/src/server.rs:18-31`)
```rust
#[derive(Debug, Clone, Builder)]
#[builder(pattern = "owned", setter(into, strip_option))]
pub struct LlamaServerArgs {
  pub model: PathBuf,
  pub alias: String,
  api_key: Option<String>,
  port: u16,
  // ...
}
```

**Command Line Arguments** (`crates/llama_server_proc/src/server.rs:51-81`)
```rust
pub fn to_args(&self) -> Vec<String> {
  let mut args = vec![
    "--alias".to_string(),
    self.alias.clone(),
    "--model".to_string(),
    self.model.to_string_lossy().to_string(),
  ];
  // Port, API key, and server args handling...
}
```

### Process Lifecycle Management

**Server Start with Health Check** (`crates/llama_server_proc/src/server.rs:212-227`)
```rust
async fn start(&self) -> Result<()> {
  let args = self.server_args.to_args();
  let mut process = Command::new(&self.executable_path)
    .args(args)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;
  
  Self::monitor_output(stdout, stderr);
  self.wait_for_server_ready().await?;
}
```

**Health Check Loop** (`crates/llama_server_proc/src/server.rs:162-186`)
```rust
async fn wait_for_server_ready(&self) -> Result<()> {
  let max_attempts = 300;
  for attempt in 0..max_attempts {
    match self.client.get(format!("{}/health", self.base_url)).send().await {
      Ok(response) if response.status().is_success() => return Ok(()),
      // Retry logic with 1-second intervals...
    }
  }
}
```

**Process Cleanup** (`crates/llama_server_proc/src/server.rs:196-208`)
```rust
impl Drop for LlamaServer {
  fn drop(&mut self) {
    let mut lock = self.process.lock().unwrap();
    if let Some(mut process) = lock.take() {
      process.kill().ok();
      process.wait().ok();
    }
  }
}
```

### HTTP Proxy Operations

**Request Proxying** (`crates/llama_server_proc/src/server.rs:188-193`)
```rust
async fn proxy_request(&self, endpoint: &str, body: &Value) -> Result<Response> {
  let url = format!("{}{}", self.base_url, endpoint);
  let response = self.client.post(url).json(body).send().await?;
  Ok(response)
}
```

**OpenAI-Compatible Endpoints** (`crates/llama_server_proc/src/server.rs:251-266`)
- Chat completions: `/v1/chat/completions`
- Embeddings: `/v1/embeddings`
- Tokenize: `/v1/tokenize`
- Detokenize: `/v1/detokenize`

### Error Handling

**ServerError Enum** (`crates/llama_server_proc/src/error.rs:5-27`)
```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
pub enum ServerError {
  ServerNotReady,
  StartupError(String),
  IoError(#[from] IoError),
  ClientError(#[from] ReqwestError),
  HealthCheckError(String),
  TimeoutError(u64),
}
```

**Localized Error Messages** (`crates/llama_server_proc/src/resources/en-US/messages.ftl:1-5`)
- server_not_ready: "server not ready: the server process has not completed initialization"
- startup_error: "failed to start server: {$var_0}"
- health_check_error: "server health check failed: {$var_0}"
- timeout_error: "server health check timed out after {$var_0} seconds"

### Build System Architecture

**Build Environment Constants** (`crates/llama_server_proc/src/build_envs.rs:1-10`)
```rust
pub static BUILD_TARGET: &str = env!("BUILD_TARGET");
pub static ref BUILD_VARIANTS: Vec<String> = {
  env!("BUILD_VARIANTS").split(',').map(String::from).collect()
};
pub static DEFAULT_VARIANT: &str = env!("DEFAULT_VARIANT");
pub static EXEC_NAME: &str = env!("EXEC_NAME");
```

**Platform-Specific Build Configuration** (`crates/llama_server_proc/build.rs:20-43`)
```rust
static LLAMA_SERVER_BUILDS: Lazy<HashSet<LlamaServerBuild>> = Lazy::new(|| {
  let mut set = HashSet::new();
  set.insert(LlamaServerBuild::new("aarch64-apple-darwin", "", vec!["metal", "cpu"]));
  set.insert(LlamaServerBuild::new("aarch64-unknown-linux-gnu", "", vec!["cpu"]));
  set.insert(LlamaServerBuild::new("x86_64-unknown-linux-gnu", "", vec!["cpu"]));
  set.insert(LlamaServerBuild::new("x86_64-pc-windows-msvc", "exe", vec!["cpu"]));
  set
});
```

**GitHub Release Download** (`crates/llama_server_proc/build.rs:286-418`)
- Asset filtering by platform and variant
- ZIP file extraction handling
- Executable permissions management
- File locking for concurrent builds (`crates/llama_server_proc/build.rs:534-555`)

### Test Utilities

**Model Fixtures** (`crates/llama_server_proc/src/test_utils/mod.rs:7-21`)
```rust
#[fixture]
pub fn llama2_7b() -> PathBuf {
  let model_path = dirs::home_dir().unwrap()
    .join(".cache/huggingface/hub/models--TheBloke--Llama-2-7B-Chat-GGUF/...");
  assert!(model_path.exists(), "Model path does not exist: {}", model_path.display());
  model_path.canonicalize().unwrap()
}
```

**HTTP Response Mocking** (`crates/llama_server_proc/src/test_utils/mod.rs:23-28`)
```rust
pub fn mock_response(body: impl Into<String>) -> Response {
  let url = Url::parse("http://127.0.0.1:8080").unwrap();
  let hyper_response = Builder::new().url(url).status(200).body(body).unwrap();
  Response::from(hyper_response)
}
```

## Usage Examples

### Basic Server Creation and Lifecycle

```rust
use llama_server_proc::{LlamaServer, LlamaServerArgsBuilder, Server};
use std::path::Path;

// Create server configuration
let args = LlamaServerArgsBuilder::default()
  .model("/path/to/model.gguf")
  .alias("my-model")
  .server_args(vec!["--ctx-size 2048".to_string()])
  .build()?;

// Create server instance
let server = LlamaServer::new(Path::new("/path/to/llama-server"), args)?;

// Start server and wait for readiness
server.start().await?;

// Use server for inference
let response = server.chat_completions(&chat_request).await?;

// Server cleanup happens automatically via Drop trait
```

### Testing with Mocks

```rust
use llama_server_proc::MockServer;
use mockall::predicate::*;

let mut mock_server = MockServer::new();
mock_server
  .expect_start()
  .times(1)
  .returning(|| Ok(()));
  
mock_server
  .expect_chat_completions()
  .with(eq(request_body))
  .returning(|_| Ok(mock_response("chat response")));
```

## Build Commands

### Development Build
```bash
# Build default variant for current platform
cargo build -p llama_server_proc

# Run tests
cargo test -p llama_server_proc

# Run integration tests (requires model files)
cargo test -p llama_server_proc --features test-utils
```

### Release Build with Binary Download
```bash
# Set CI environment for binary download
export CI_RELEASE=true
export GH_PAT=your_github_token
export CI_BUILD_TARGET=aarch64-apple-darwin
export CI_BUILD_VARIANTS=metal,cpu
export CI_DEFAULT_VARIANT=metal

cargo build -p llama_server_proc --release
```

### Cross-Platform Build
```bash
# Build for specific target
make build-aarch64-apple-darwin-metal

# Clean build artifacts
make clean
```

## Configuration Options

### Server Arguments
- `--ctx-size N`: Context window size
- `--parallel N`: Number of parallel requests
- `--batch-size N`: Batch size for processing
- `--threads N`: Number of CPU threads
- `--gpu-layers N`: Number of GPU layers (for GPU variants)

### Environment Variables
- `CI_RELEASE`: Enable release build with binary download
- `GH_PAT`: GitHub personal access token for release downloads
- `CI_BUILD_TARGET`: Target platform for CI builds
- `CI_BUILD_VARIANTS`: Comma-separated list of acceleration variants
- `CI_DEFAULT_VARIANT`: Default acceleration variant to use

### Build Variants
- `cpu`: CPU-only inference
- `metal`: Apple Metal GPU acceleration (macOS only)
- `cuda`: NVIDIA CUDA acceleration (Linux only)

## Integration Points

### With objs Crate
- Error handling via `ServerError` implementing `AppError`
- Builder pattern validation using `BuilderError`
- Localized error messages through FluentLocalizationService

### With Higher-Level Services
- Trait-based abstraction enables dependency injection
- Async interface supports tokio-based service architecture
- HTTP response streaming for real-time inference

### With Build System
- Makefile integration for cross-platform builds
- GitHub release coordination for binary distribution
- File locking prevents concurrent build conflicts