use axum::Router;
use objs::{AppError, ErrorType, LogLevel};
use server_app::{ServeCommand, ServeError, ServerShutdownHandle};
use services::{AppService, BODHI_EXEC_LOOKUP_PATH, BODHI_LOGS, BODHI_LOG_STDOUT};
use std::sync::{Arc, Mutex};
use tauri::{
  menu::{Menu, MenuEvent, MenuItem},
  path::BaseDirectory,
  tray::TrayIconBuilder,
  AppHandle, Manager, Window, WindowEvent,
};

pub struct NativeCommand {
  service: Arc<dyn AppService>,
  ui: bool,
}

type ServerHandleState = Arc<Mutex<Option<ServerShutdownHandle>>>;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum NativeError {
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, code = "tauri",  args_delegate = false)]
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
    let app_service = self.service.clone();
    let env_service = self.service.env_service();
    let ui = self.ui;

    let log_level: LogLevel = env_service.log_level();
    let mut log_plugin = tauri_plugin_log::Builder::default()
      .level(log_level)
      .max_file_size(50_000)
      .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll);
    let setting_service = self.service.env_service().setting_service();
    if let Some(serde_yaml::Value::Bool(true)) = setting_service.get_setting_value(BODHI_LOG_STDOUT)
    {
      log_plugin = log_plugin.target(tauri_plugin_log::Target::new(
        tauri_plugin_log::TargetKind::Stdout,
      ));
    }
    if let Some(bodhi_logs) = setting_service.get_setting(BODHI_LOGS) {
      log_plugin = log_plugin.target(tauri_plugin_log::Target::new(
        tauri_plugin_log::TargetKind::Folder {
          path: std::path::PathBuf::from(bodhi_logs),
          file_name: None,
        },
      ));
    }
    let log_plugin = log_plugin.build();
    tauri::Builder::default()
      .plugin(log_plugin)
      .setup(move |app| {
        #[cfg(target_os = "macos")]
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);

        let bodhi_exec_lookup_path = app.path().resolve("bin", BaseDirectory::Resource)?;
        env_service.setting_service().set_default(
          BODHI_EXEC_LOOKUP_PATH,
          &serde_yaml::Value::String(bodhi_exec_lookup_path.display().to_string())
        );
        let host = env_service.host();
        let port = env_service.port();
        let addr = env_service.server_url();
        let cmd = ServeCommand::ByParams { host, port };
        let shared_server_handle: Arc<Mutex<Option<ServerShutdownHandle>>> = Arc::new(Mutex::new(None));
        app.manage(shared_server_handle.clone());
        tokio::spawn(async move {
          match cmd
          .get_server_handle(app_service, static_router)
          .await {
            Ok(server_handle) => {shared_server_handle.lock().unwrap().replace(server_handle);},
            Err(err) => {
              tracing::error!(?err, "failed to start the backend server");
            }
          }
        });
        let homepage = MenuItem::with_id(app, "homepage", "Open Homepage", true, None::<&str>)?;
        let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
        let menu = Menu::with_items(app, &[&homepage, &quit])?;
        TrayIconBuilder::new()
          .menu(&menu)
          .menu_on_left_click(true)
          .icon(app.default_window_icon().unwrap().clone())
          .on_menu_event(move |app, event| {
            on_menu_event(app, event, &addr);
          })
          .build(app)?;

        // Attempt to open the default web browser
        if ui {
          if let Err(err) = webbrowser::open(env_service.server_url().as_str()) {
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
