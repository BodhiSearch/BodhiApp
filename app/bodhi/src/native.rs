use bodhicore::{
  bindings::{disable_llama_log, llama_server_disable_logging},
  server::{build_routes, build_server_handle, Server, ServerHandle, ServerParams},
  AppService, SharedContextRw, SharedContextRwFn,
};
use futures_util::{future::BoxFuture, FutureExt};
use std::sync::{Arc, Mutex};
use tauri::{
  AppHandle, CustomMenuItem, Manager, RunEvent, SystemTray, SystemTrayEvent, SystemTrayMenu,
  WindowEvent,
};
use tokio::{
  runtime::Builder,
  sync::oneshot::{self, Receiver, Sender},
  task::JoinHandle,
};

pub(super) fn main_native() -> anyhow::Result<()> {
  let runtime = Builder::new_multi_thread().enable_all().build();
  match runtime {
    Ok(runtime) => runtime.block_on(async move { _main_native().await }),
    Err(err) => Err(err.into()),
  }
}

async fn _main_native() -> anyhow::Result<()> {
  let system_tray = SystemTray::new().with_menu(
    SystemTrayMenu::new()
      .add_item(CustomMenuItem::new("homepage", "Open Homepage"))
      .add_item(CustomMenuItem::new("quit".to_string(), "Quit")),
  );
  tauri::Builder::default()
    .setup(|app| {
      #[cfg(target_os = "macos")]
      app.set_activation_policy(tauri::ActivationPolicy::Accessory);
      // launch the web server
      let result = launch_server();
      if let Err(err) = result {
        tracing::error!(err = format!("{err}"), "failed to start the webserver");
        std::process::exit(1);
      }
      let server_state = result.unwrap();
      app.manage(server_state);
      // Attempt to open the default web browser at localhost:1135
      if let Err(err) = webbrowser::open("http://localhost:1135/") {
        tracing::info!(err=?err, "failed to open browser");
      }
      Ok(())
    })
    .system_tray(system_tray)
    .on_system_tray_event(on_system_tray_event)
    .on_window_event(|event| {
      if let WindowEvent::CloseRequested { api, .. } = event.event() {
        event.window().hide().unwrap();
        api.prevent_close();
      }
    })
    .build(tauri::generate_context!())?
    .run(|_app_handle, event| {
      if let RunEvent::ExitRequested { api, .. } = event {
        api.prevent_exit();
      }
    });
  Ok(())
}

fn on_system_tray_event(app: &AppHandle, event: SystemTrayEvent) {
  if let SystemTrayEvent::MenuItemClick { id, .. } = event {
    match id.as_str() {
      "homepage" => {
        webbrowser::open("http://localhost:1135/").expect("should not fail to open homepage");
      }
      "quit" => {
        let state = app.state::<ServerState>();
        // TODO - move shutdown and wait to ServerState
        if let Some(shutdown) = state.take() {
          tracing::info!("sending shutdown signal");
          if shutdown.send(()).is_ok() {
            tracing::info!("shutdown signal sent successfully");
          } else {
            tracing::info!("error sending shutdown signal");
          }
        } else {
          tracing::info!("shutdown channel missing");
        }
        let handle = state.handle.lock().unwrap().take().unwrap();
        let app_clone = app.clone();
        tokio::spawn(async move {
          match handle.await {
            Err(err) => {
              tracing::warn!(err = err.to_string(), "server stopped with error");
            }
            Ok(result) => match result {
              Ok(()) => {
                tracing::info!("server closed successfully");
              }
              Err(err) => {
                tracing::info!(err=?err,"server stopped with error")
              }
            },
          };
          app_clone.exit(0);
        });
      }
      _ => {}
    }
  }
}

fn launch_server() -> anyhow::Result<ServerState> {
  main_server(ServerParams::default())
}

fn main_server(server_params: ServerParams) -> anyhow::Result<ServerState> {
  let ServerHandle {
    server,
    shutdown,
    ready_rx,
  } = build_server_handle(server_params)?;
  let server_async = tokio::spawn(async move { start_server(server, ready_rx).await });
  Ok(ServerState::new(server_async, shutdown))
}

async fn start_server(server: Server, ready_rx: Receiver<()>) -> anyhow::Result<()> {
  disable_llama_log();
  unsafe {
    llama_server_disable_logging();
  }
  let ctx = SharedContextRw::new_shared_rw(None).await?;
  let ctx = Arc::new(ctx);
  let app_service = AppService::default();
  let app = build_routes(ctx.clone(), Arc::new(app_service));
  let callback: Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send + 'static> = Box::new(|| {
    async move {
      if let Err(err) = ctx.try_stop().await {
        tracing::warn!(err = ?err, "error stopping llama context");
      }
    }
    .boxed()
  });
  if let Err(err) = server.start_new(app, Some(callback)).await {
    tracing::error!(err = ?err, "server startup resulted in an error");
    return Err(err);
  }
  if let Err(err) = ready_rx.await {
    tracing::warn!(err = ?err, "server ready status received as error");
  }
  Ok(())
}

struct ServerState {
  handle: Mutex<Option<JoinHandle<Result<(), anyhow::Error>>>>,
  shutdown: Mutex<Option<Sender<()>>>,
}

impl ServerState {
  fn new(handle: JoinHandle<Result<(), anyhow::Error>>, shutdown: Sender<()>) -> Self {
    ServerState {
      handle: Mutex::new(Some(handle)),
      shutdown: Mutex::new(Some(shutdown)),
    }
  }

  fn take(&self) -> Option<oneshot::Sender<()>> {
    self.shutdown.lock().unwrap().take()
  }
}
