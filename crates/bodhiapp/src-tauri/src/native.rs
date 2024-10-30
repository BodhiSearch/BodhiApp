use axum::Router;
use include_dir::{include_dir, Dir};
use objs::{AppError, ErrorType, LogLevel};
use server_app::{ServeCommand, ServeError, ServerShutdownHandle};
use services::AppService;
use std::sync::{Arc, Mutex};
use tauri::{
  menu::{Menu, MenuEvent, MenuItem},
  tray::TrayIconBuilder,
  AppHandle, Manager, Window, WindowEvent,
};
use tower_serve_static::ServeDir;

static ASSETS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../out");

pub fn static_router() -> Router {
  let static_service = ServeDir::new(&ASSETS).append_index_html_on_directories(true);
  Router::new().fallback_service(static_service)
}

pub struct NativeCommand {
  service: Arc<dyn AppService>,
  ui: bool,
}

type ServerHandleState = Arc<Mutex<Option<ServerShutdownHandle>>>;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum NativeError {
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "tauri",  args_delegate = false)]
  Tauri(#[from] tauri::Error),
  #[error(transparent)]
  Serve(#[from] ServeError),
}

type Result<T> = std::result::Result<T, NativeError>;

impl NativeCommand {
  pub fn new(service: Arc<dyn AppService>, ui: bool) -> Self {
    Self { service, ui }
  }

  // TODO: modbile entry point as marked by default tauri app generator
  // #[cfg_attr(mobile, tauri::mobile_entry_point)]
  pub async fn aexecute(&self, static_router: Option<Router>) -> Result<()> {
    let addr = self.service.env_service().server_url();
    let addr_clone = addr.clone();
    let cmd = ServeCommand::ByParams { host, port };
    let server_handle = cmd
      .get_server_handle(self.service.clone(), static_router)
      .await?;
    let ui = self.ui;
    let log_level: LogLevel = self.service.env_service().log_level();

    tauri::Builder::default()
      .setup(move |app| {
        if cfg!(debug_assertions) {
          app.handle().plugin(
            tauri_plugin_log::Builder::default()
              .level(log_level)
              .build(),
          )?;
        }
        let homepage = MenuItem::with_id(app, "homepage", "Open Homepage", true, None::<&str>)?;
        let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
        let menu = Menu::with_items(app, &[&homepage, &quit])?;
        TrayIconBuilder::new()
          .menu(&menu)
          .menu_on_left_click(true)
          .icon(app.default_window_icon().unwrap().clone())
          .on_menu_event(move |app, event| {
            on_menu_event(app, event, &addr_clone);
          })
          .build(app)?;

        #[cfg(target_os = "macos")]
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);

        app.manage(Arc::new(Mutex::new(Some(server_handle))));
        // Attempt to open the default web browser
        if ui {
          if let Err(err) = webbrowser::open(&addr) {
            tracing::info!(?err, "failed to open browser");
          }
        }
        Ok(())
      })
      .on_window_event(on_window_event)
      .run(tauri::generate_context!())?
      // .run(|_app_handle, event| {
      //   if let RunEvent::ExitRequested { api, .. } = event {
      //     api.prevent_exit();
      //   }
      // })
      ;
    Ok(())
  }
}

fn on_window_event(window: &Window, event: &WindowEvent) {
  if let WindowEvent::CloseRequested { api, .. } = event {
    window.hide().unwrap();
    api.prevent_close();
  }
}

fn on_menu_event(app: &AppHandle, event: MenuEvent, addr: &str) {
  match event.id.as_ref() {
    "homepage" => {
      webbrowser::open(addr).expect("should not fail to open homepage");
    }
    "quit" => {
      let server_handle = app.state::<ServerHandleState>();
      let guard_result = server_handle.lock();
      let app_clone = app.clone();
      match guard_result {
        Ok(mut guard) => {
          let handle = guard.take();
          if let Some(handle) = handle {
            tokio::spawn(async move {
              if let Err(err) = handle.shutdown().await {
                tracing::warn!(?err, "error on server shutdown");
                app_clone.exit(1);
              } else {
                app_clone.exit(0);
              }
            });
          } else {
            tracing::warn!("cannot find server handle in app state");
            app_clone.exit(1);
          }
        }
        Err(err) => {
          tracing::warn!(?err, "error acquiring server shutdown instance");
          app_clone.exit(1);
        }
      }
    }
    &_ => {}
  }
}
