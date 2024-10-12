use axum::Router;
use server_app::{ServeCommand, ServerShutdownHandle};
use services::AppService;
use std::sync::{Arc, Mutex};
use tauri::{
  AppHandle, CustomMenuItem, Manager, RunEvent, SystemTray, SystemTrayEvent, SystemTrayMenu,
  WindowEvent,
};

pub struct NativeCommand {
  service: Arc<dyn AppService>,
  ui: bool,
}

type ServerHandleState = Arc<Mutex<Option<ServerShutdownHandle>>>;

impl NativeCommand {
  pub fn new(service: Arc<dyn AppService>, ui: bool) -> Self {
    Self { service, ui }
  }

  pub async fn aexecute(&self, static_router: Option<Router>) -> crate::error::Result<()> {
    let host = self.service.env_service().host();
    let port = self.service.env_service().port();
    let addr = format!("http://{host}:{port}/");
    let addr_clone = addr.clone();
    let cmd = ServeCommand::ByParams { host, port };
    let server_handle = cmd
      .get_server_handle(self.service.clone(), static_router)
      .await?;
    let ui = self.ui;

    let system_tray = SystemTray::new().with_menu(
      SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("homepage", "Open Homepage"))
        .add_item(CustomMenuItem::new("quit".to_string(), "Quit")),
    );
    tauri::Builder::default()
      .setup(move |app| {
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
      .system_tray(system_tray)
      .on_system_tray_event(move |app: &AppHandle, event: SystemTrayEvent| {
        on_system_tray_event(app, event, &addr_clone);
      })
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
}

fn on_system_tray_event(app: &AppHandle, event: SystemTrayEvent, addr: &str) {
  if let SystemTrayEvent::MenuItemClick { id, .. } = event {
    match id.as_str() {
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
      _ => {}
    }
  }
}
