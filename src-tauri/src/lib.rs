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
use tauri_plugin_positioner::{WindowExt, Position};

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
    // 1. Find (or create) the user config dir
    let config_dir = app.path().app_config_dir()
        .map_err(|e| anyhow::anyhow!("failed to get app_config_dir: {e}"))?;
    fs::create_dir_all(&config_dir)
        .map_err(|e| anyhow::anyhow!("failed to create config_dir {config_dir:?}: {e}"))?;

    // Target path inside config dir
    let json_path = config_dir.join("apps.json");

    // 2. If user config doesn’t exist yet, seed it
    if !json_path.exists() {
        // Try bundled resource first
        let resource_path: PathBuf = match app.path().resolve("apps.json", tauri::path::BaseDirectory::Resource) {
            Ok(path) => path,
            Err(_) => PathBuf::from("src-tauri/resources/apps.json"), // fallback for dev
        };

        if resource_path.exists() {
            fs::copy(&resource_path, &json_path)
                .map_err(|e| anyhow::anyhow!("failed to copy from {resource_path:?} to {json_path:?}: {e}"))?;
            println!("Seeded apps.json from {:?}", resource_path);
        } else {
            // Last fallback: just create empty JSON
            fs::write(&json_path, "[]")?;
            println!("apps.json not found in resources, created empty file at {:?}", json_path);
        }
    }

    Ok(json_path)
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.set_focus();
            }
        }))
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            if let Some(win) = app.get_webview_window("main") {
                let _ = win.move_window(Position::BottomCenter);
            }

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

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet,get_apps])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
