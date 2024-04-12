// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
  AppHandle, CustomMenuItem, RunEvent, SystemTray, SystemTrayEvent, SystemTrayMenu, WindowEvent,
};

fn main() -> anyhow::Result<()> {
  let system_tray = SystemTray::new().with_menu(
    SystemTrayMenu::new()
      .add_item(CustomMenuItem::new("homepage", "Open Homepage"))
      .add_item(CustomMenuItem::new("quit".to_string(), "Quit")),
  );

  tauri::Builder::default()
    .system_tray(system_tray)
    .on_system_tray_event(on_system_tray_event)
    .on_window_event(|event| {
      if let WindowEvent::CloseRequested { api, .. } = event.event() {
        event.window().hide().unwrap();
        api.prevent_close();
      }
    })
    .setup(|app| {
      #[cfg(target_os = "macos")]
      app.set_activation_policy(tauri::ActivationPolicy::Accessory);
      // Attempt to open the default web browser at localhost:7735
      if let Err(e) = webbrowser::open("http://localhost:7735/app") {
        eprintln!("Failed to open browser: {}", e);
      }
      Ok(())
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
        webbrowser::open("http://localhost:7735/app").expect("should not fail to open homepage");
      }
      "quit" => {
        app.exit(0);
      }
      _ => {}
    }
  }
}
