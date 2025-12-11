// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tauri::{Manager};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use sha2::{Sha384, Digest};
use std::path::Path;
use std::os::windows::prelude::OsStrExt;

use tauri_plugin_positioner::{WindowExt, Position};
use tauri::AppHandle;
use anyhow::{anyhow, Result};

use windows::{
    core::PCWSTR,
    Win32::{
        UI::{
            Shell::ExtractIconExW,
            WindowsAndMessaging::{DestroyIcon,HICON}
        },
    },
};


#[derive(Debug, Serialize, Deserialize)]
pub struct AppEntry {
    pub name: String,
    pub exe: String,
    pub publisher: String,
    pub icon: Option<String>, // null in JSON → None in Rust
}

fn extract_icons(path: &std::path::Path) -> anyhow::Result<()> {
    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut large_icon: HICON = HICON(std::ptr::null_mut());
    let mut small_icon: HICON = HICON(std::ptr::null_mut());

    let count = unsafe {
        ExtractIconExW(
            PCWSTR(wide.as_ptr()),
            0,
            Some(&mut large_icon),
            Some(&mut small_icon),
            1,
        )
    };

    if count == 0 {
        anyhow::bail!("No icons found");
    }

    // ... use large_icon or small_icon ...

    unsafe {
        if large_icon.0 != std::ptr::null_mut() {
            DestroyIcon(large_icon);
        }
        if small_icon.0 != std::ptr::null_mut() {
            DestroyIcon(small_icon);
        }
    }

    Ok(())
}

/// Load the working `apps.json` into a Vec<AppEntry>.
pub fn load_apps(json_path: &PathBuf) -> anyhow::Result<Vec<AppEntry>> {
    let data = fs::read_to_string(json_path)?;
    let apps: Vec<AppEntry> = serde_json::from_str(&data)?;
    Ok(apps)
}

pub fn hash_app_path(path: &Path) -> String {
    // Normalize for consistency (lowercase + absolute path)
    let normalized = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf()) // fallback if file doesn't exist
        .to_string_lossy()
        .to_lowercase();

    // Compute SHA-384 digest
    let mut hasher = Sha384::new();
    hasher.update(normalized.as_bytes());
    let digest = hasher.finalize();

    // Convert to hex string
    hex::encode(digest)
}

pub fn ensure_icocache_dir(app: &AppHandle) -> Result<PathBuf> {
    // Base directory managed by Tauri (e.g., AppData\Roaming\<app>)
    let base_dir = app.path().app_config_dir()?;
    let cache_dir = base_dir.join("icocache");

    // Create it if missing
    fs::create_dir_all(&cache_dir)?;

    Ok(cache_dir)
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
            // handle cache dir
            let cache_dir = ensure_icocache_dir(&app.handle())?;
            println!("Icon cache directory: {}", cache_dir.display());
            
            // temporary code
            let sample_path = std::path::Path::new("C:\\Windows\\notepad.exe");
            println!("Hash: {}", hash_app_path(sample_path));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet,get_apps])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
