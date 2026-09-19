use super::runtime::{
  ConnectorHandle, ConnectorProcess, HttpMethod, HttpRequest, HttpResponse, OutputStream,
  RawOutput, TunnelIo, TunnelRuntimeError,
};
#[cfg(unix)]
use std::os::{fd::OwnedFd, unix::net::UnixStream};
use std::{
  io::{BufRead, BufReader},
  path::{Path, PathBuf},
  process::{Child, Command, Stdio},
  sync::mpsc::{channel, Receiver, Sender},
  thread,
  time::Duration,
};
use tracing::warn;

fn io_error(error: impl std::fmt::Display) -> TunnelRuntimeError {
  TunnelRuntimeError::Io(error.to_string())
}

#[derive(Debug, Default)]
pub struct SystemCloudflaredCli;

#[async_trait::async_trait]
impl super::runtime::CloudflaredCli for SystemCloudflaredCli {
  async fn run(
    &self,
    binary: PathBuf,
    args: Vec<String>,
    envs: Vec<(String, String)>,
    timeout: Duration,
  ) -> Result<RawOutput, TunnelRuntimeError> {
    let mut command = tokio::process::Command::new(binary);
    command.args(args).kill_on_drop(true);
    for (key, value) in envs {
      command.env(key, value);
    }
    match tokio::time::timeout(timeout, command.output()).await {
      Ok(Ok(output)) => Ok(RawOutput {
        success: output.status.success(),
        code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
      }),
      Ok(Err(error)) => Err(io_error(error)),
      Err(_) => Err(TunnelRuntimeError::Timeout),
    }
  }
}

#[derive(Debug, Default)]
pub struct SystemConnectorProcess;

#[derive(Debug)]
pub struct SystemConnectorHandle {
  child: Child,
  #[cfg(unix)]
  liveness: Option<UnixStream>,
  output: Option<Receiver<(OutputStream, String)>>,
}

fn forward_lines(
  reader: impl std::io::Read + Send + 'static,
  stream: OutputStream,
  sender: Sender<(OutputStream, String)>,
) {
  thread::spawn(move || {
    for line in BufReader::new(reader)
      .lines()
      .map_while(std::result::Result::ok)
    {
      if sender.send((stream, line)).is_err() {
        return;
      }
    }
  });
}

impl ConnectorProcess for SystemConnectorProcess {
  fn spawn(
    &self,
    binary: PathBuf,
    args: Vec<String>,
    envs: Vec<(String, String)>,
  ) -> Result<Box<dyn ConnectorHandle>, TunnelRuntimeError> {
    #[cfg(unix)]
    let (liveness_read, liveness_write) = UnixStream::pair().map_err(io_error)?;
    #[cfg(unix)]
    let mut command = {
      let read_fd: OwnedFd = liveness_read.into();
      let mut command = Command::new("/bin/sh");
      command
        .args([
          "-c",
          "exec 3<&0; \"$@\" </dev/null & child=$!; ( read -r _ <&3 || true; kill \"$child\" 2>/dev/null || true ) & reader=$!; trap 'kill \"$child\" 2>/dev/null || true' INT TERM; wait \"$child\"; status=$?; kill \"$reader\" 2>/dev/null || true; exit \"$status\"",
          "--",
        ])
        .arg(&binary)
        .args(&args)
        .stdin(Stdio::from(read_fd));
      command
    };
    #[cfg(not(unix))]
    let mut command = {
      let mut command = Command::new(binary);
      command.args(&args);
      command
    };
    for (key, value) in envs {
      command.env(key, value);
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().map_err(io_error)?;
    let (sender, receiver) = channel();
    if let Some(stdout) = child.stdout.take() {
      forward_lines(stdout, OutputStream::Stdout, sender.clone());
    }
    if let Some(stderr) = child.stderr.take() {
      forward_lines(stderr, OutputStream::Stderr, sender);
    }
    Ok(Box::new(SystemConnectorHandle {
      child,
      #[cfg(unix)]
      liveness: Some(liveness_write),
      output: Some(receiver),
    }))
  }
}

impl ConnectorHandle for SystemConnectorHandle {
  fn try_wait(&mut self) -> Result<Option<Option<i32>>, TunnelRuntimeError> {
    self
      .child
      .try_wait()
      .map(|status| status.map(|status| status.code()))
      .map_err(io_error)
  }

  fn stop(&mut self) -> Result<(), TunnelRuntimeError> {
    #[cfg(unix)]
    {
      self.liveness.take();
    }
    if self.child.try_wait().map_err(io_error)?.is_some() {
      return Ok(());
    }
    #[cfg(not(unix))]
    self.child.kill().map_err(io_error)?;
    self.child.wait().map_err(io_error)?;
    Ok(())
  }

  fn take_output(&mut self) -> Option<Receiver<(OutputStream, String)>> {
    self.output.take()
  }
}

#[derive(Debug)]
pub struct SystemTunnelIo {
  client: reqwest::Client,
}

impl Default for SystemTunnelIo {
  fn default() -> Self {
    Self {
      client: reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap_or_default(),
    }
  }
}

#[async_trait::async_trait]
impl TunnelIo for SystemTunnelIo {
  async fn read_to_string(&self, path: PathBuf) -> Result<String, TunnelRuntimeError> {
    std::fs::read_to_string(path).map_err(io_error)
  }

  fn is_file(&self, path: &Path) -> bool {
    path.is_file()
  }

  fn home_dir(&self) -> Option<PathBuf> {
    dirs::home_dir()
  }

  fn path_candidates(&self) -> Vec<PathBuf> {
    let executable = if cfg!(windows) {
      "cloudflared.exe"
    } else {
      "cloudflared"
    };
    std::env::var_os("PATH")
      .map(|path| {
        std::env::split_paths(&path)
          .map(|directory| directory.join(executable))
          .collect()
      })
      .unwrap_or_default()
  }

  fn standard_locations(&self) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    #[cfg(target_os = "macos")]
    paths.extend([
      PathBuf::from("/opt/homebrew/bin/cloudflared"),
      PathBuf::from("/usr/local/bin/cloudflared"),
    ]);
    #[cfg(target_os = "linux")]
    paths.extend([
      PathBuf::from("/usr/local/bin/cloudflared"),
      PathBuf::from("/usr/bin/cloudflared"),
    ]);
    #[cfg(windows)]
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
      paths.push(PathBuf::from(program_files).join("cloudflared/cloudflared.exe"));
    }
    paths
  }

  async fn prepare_scratch_dir(&self, dir: PathBuf) -> Result<(), TunnelRuntimeError> {
    std::fs::create_dir_all(&dir).map_err(io_error)?;
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).map_err(io_error)?;
    }
    Ok(())
  }

  async fn discard_scratch(&self, path: PathBuf) {
    #[cfg(unix)]
    if path.is_file() {
      use std::os::unix::fs::PermissionsExt;
      let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
    if let Err(err) = std::fs::remove_file(&path) {
      if err.kind() != std::io::ErrorKind::NotFound {
        warn!(?err, "failed to remove tunnel credentials scratch file");
      }
    }
  }

  async fn http(&self, request: HttpRequest) -> Result<HttpResponse, TunnelRuntimeError> {
    let mut builder = match request.method {
      HttpMethod::Get => self.client.get(&request.url),
      HttpMethod::Delete => self.client.delete(&request.url),
    };
    if let Some(bearer) = request.bearer {
      builder = builder.bearer_auth(bearer);
    }
    if !request.query.is_empty() {
      builder = builder.query(&request.query);
    }
    let response = builder.send().await.map_err(io_error)?;
    let status = response.status().as_u16();
    let body = response.text().await.map_err(io_error)?;
    Ok(HttpResponse { status, body })
  }
}
