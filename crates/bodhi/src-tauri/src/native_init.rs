use crate::app::AppCommand;
use crate::common::build_app_options;
use lib_bodhiserver::{
  build_app_service, setup_app_dirs, setup_bootstrap_service, AppError, AppService, AppType,
  ErrorMessage, ErrorType, LogLevel, ServeCommand, ServeError, ServerShutdownHandle,
  BODHI_EXEC_LOOKUP_PATH, BODHI_LOGS, BODHI_LOG_STDOUT,
};
use std::sync::{Arc, Mutex};
use tauri::{
  menu::{Menu, MenuEvent, MenuItem},
  path::BaseDirectory,
  tray::TrayIconBuilder,
  AppHandle, Manager, Window, WindowEvent,
};
use tokio::runtime::Builder;

const APP_TYPE: AppType = AppType::Native;

pub struct NativeCommand {
  service: Arc<dyn AppService>,
  ui: bool,
}

type ServerHandleState = Arc<Mutex<Option<ServerShutdownHandle>>>;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum NativeError {
  #[error("Desktop application error: {0}.")]
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
  pub async fn aexecute(
    &self,
    static_dir: Option<&'static include_dir::Dir<'static>>,
  ) -> Result<()> {
    let app_service = self.service.clone();
    let setting_service = self.service.setting_service();
    let ui = self.ui;

    // Native mode reads log config from async SettingService (can access DB), unlike
    // server/NAPI modes which read from BootstrapService (env + yaml only). This is
    // intentional: tauri configures logging inside the async context.
    let log_level: LogLevel = setting_service.log_level().await;
    let mut log_plugin = tauri_plugin_log::Builder::default()
      .level(log_level)
      .max_file_size(50_000)
      .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll);
    if let Some(serde_yaml::Value::Bool(true)) =
      setting_service.get_setting_value(BODHI_LOG_STDOUT).await
    {
      log_plugin = log_plugin.target(tauri_plugin_log::Target::new(
        tauri_plugin_log::TargetKind::Stdout,
      ));
    }
    if let Some(bodhi_logs) = setting_service.get_setting(BODHI_LOGS).await {
      log_plugin = log_plugin.target(tauri_plugin_log::Target::new(
        tauri_plugin_log::TargetKind::Folder {
          path: std::path::PathBuf::from(bodhi_logs),
          file_name: None,
        },
      ));
    }
    let log_plugin = log_plugin.build();

    let host = setting_service.host().await;
    let port = setting_service.port().await;
    let addr = setting_service.public_server_url().await;
    let browser_url = addr.clone();

    tauri::Builder::default()
      .plugin(log_plugin)
      .setup(move |app| {
        #[cfg(target_os = "macos")]
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);

        let bodhi_exec_lookup_path = app.path().resolve("bin", BaseDirectory::Resource)?;
        // block_in_place requires a multi-threaded tokio runtime (created in initialize_and_execute)
        tokio::task::block_in_place(|| {
          tokio::runtime::Handle::current().block_on(async {
            setting_service
              .set_default(
                BODHI_EXEC_LOOKUP_PATH,
                &serde_yaml::Value::String(bodhi_exec_lookup_path.display().to_string()),
              )
              .await
          })
        })?;
        let cmd = ServeCommand::ByParams { host, port };
        let shared_server_handle: Arc<Mutex<Option<ServerShutdownHandle>>> =
          Arc::new(Mutex::new(None));
        app.manage(shared_server_handle.clone());
        tokio::spawn(async move {
          match cmd.get_server_handle(app_service, static_dir).await {
            Ok(server_handle) => {
              shared_server_handle.lock().unwrap().replace(server_handle);
            }
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
          .show_menu_on_left_click(true)
          .icon(app.default_window_icon().unwrap().clone())
          .on_menu_event(move |app, event| {
            on_menu_event(app, event, &addr);
          })
          .build(app)?;

        if ui {
          if let Err(err) = webbrowser::open(browser_url.as_str()) {
            tracing::info!(?err, "failed to open browser");
          }
        }
        Ok(())
      })
      .on_window_event(on_window_event)
      .run(tauri::generate_context!())?;
    Ok(())
  }
}

fn on_window_event(window: &Window, event: &WindowEvent) {
  if let WindowEvent::CloseRequested { api, .. } = event {
    if let Err(err) = window.hide() {
      tracing::warn!(?err, "error hiding window");
    }
    api.prevent_close();
  }
}

fn on_menu_event(app: &AppHandle, event: MenuEvent, addr: &str) {
  match event.id.as_ref() {
    "homepage" => {
      if let Err(err) = webbrowser::open(addr) {
        tracing::warn!(?err, "error opening browser");
      }
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

pub fn initialize_and_execute(_command: AppCommand) -> std::result::Result<(), ErrorMessage> {
  let app_options = build_app_options(APP_TYPE)?;
  let (bodhi_home, source, file_defaults) = setup_app_dirs(&app_options)?;
  let bootstrap = setup_bootstrap_service(
    &app_options,
    bodhi_home,
    source,
    file_defaults,
    AppCommand::Default,
  )?;
  let parts = bootstrap.into_parts();

  let runtime = Builder::new_multi_thread()
    .enable_all()
    .build()
    .map_err(crate::error::AppSetupError::from)?;
  let result: std::result::Result<(), ErrorMessage> = runtime.block_on(async move {
    let app_service = Arc::new(build_app_service(parts).await?);

    match NativeCommand::new(app_service, true)
      .aexecute(Some(&crate::ui::ASSETS))
      .await
    {
      Err(err) => {
        tracing::warn!(?err, "application exited with error");
        let err_msg = ErrorMessage::new(
          "native_error".to_string(),
          ErrorType::InternalServer.to_string(),
          err.to_string(),
        );
        Err(err_msg)
      }
      Ok(_) => {
        tracing::info!("application exited with success");
        Ok(())
      }
    }
  });
  result
}
