// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tauri::{Manager};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppEntry {
    pub name: String,
    pub exe: String,
    pub publisher: String,
    pub icon: Option<String>, // null in JSON → None in Rust
}

/// Load the working `apps.json` into a Vec<AppEntry>.
pub fn load_apps(json_path: &PathBuf) -> anyhow::Result<Vec<AppEntry>> {
    let data = fs::read_to_string(json_path)?;
    let apps: Vec<AppEntry> = serde_json::from_str(&data)?;
    Ok(apps)
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_apps(app: tauri::AppHandle) -> Result<Vec<AppEntry>, String> {
    let json_path = ensure_apps_json(&app).map_err(|e| e.to_string())?;
    let apps = load_apps(&json_path).map_err(|e| e.to_string())?;
    Ok(apps)
}

fn ensure_apps_json(app: &tauri::AppHandle) -> anyhow::Result<PathBuf> {
    // 1. Find the app config dir (OS-specific, managed by Tauri)
    let config_dir = app.path().app_config_dir()?;
    fs::create_dir_all(&config_dir)?;

    // Path to working JSON
    let json_path = config_dir.join("apps.json");

    // 2. If missing, copy from bundled resources
    if !json_path.exists() {
        let resource_path = app
            .path()
            .resolve("resources/apps.json", tauri::path::BaseDirectory::Resource)?;
        fs::copy(&resource_path, &json_path)?;
        println!("Copied default apps.json to {:?}", json_path);
    }

    Ok(json_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();
    tauri::Builder::default()
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;
            let tray = TrayIconBuilder::new()

            .menu(&menu)
            .show_menu_on_left_click(false)
            .on_menu_event(|app, event| match event.id.as_ref() {
                "quit" => {
                    println!("quit menu item was clicked");
                        app.exit(0);
                    }
                    _ => {
                        println!("menu item {:?} not handled", event.id);
                    }
            })


            .icon(app.default_window_icon().unwrap().clone())
            .build(app)?;
            // let handle = app.handle();
            let json_path = ensure_apps_json(&app.handle())?;

            println!("Using apps.json at: {}", json_path.display());



            #[cfg(desktop)]
                {
                    builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
                    let _ = app.get_webview_window("main")
                       .expect("no main window")
                       .set_focus();
                }));
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet,get_apps])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
