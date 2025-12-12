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
use std::ffi::c_void; // Import c_void
use windows::Win32::Graphics::Gdi::{GetDC, ReleaseDC, DeleteObject}; // Import GetDC and DeleteObject

use tauri_plugin_positioner::{WindowExt, Position};
use tauri::AppHandle;
use anyhow::{anyhow, Result};

use windows::{
    core::PCWSTR,
    Win32::{
        UI::{
            Shell::ExtractIconExW,
            WindowsAndMessaging::{DestroyIcon,HICON, GetIconInfo, ICONINFO}
        },
        Graphics::Gdi::{
            GetObjectW, GetDIBits,
            BITMAP, BITMAPINFO, BITMAPINFOHEADER,
            DIB_RGB_COLORS,
            BI_RGB
        },
        Foundation::{HWND}
    },
    core::Error as WinError,
};
use image::{RgbaImage, ImageEncoder};
use image::codecs::png::PngEncoder;

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

fn hicon_to_png_bytes(icon: HICON) -> anyhow::Result<Vec<u8>> {
    unsafe {
        //
        // 1. Read ICONINFO
        //
        let mut info = ICONINFO::default();
        if !GetIconInfo(icon, &mut info).is_ok() {
            return Err(anyhow::anyhow!("GetIconInfo failed"));
        }

        //
        // 2. Read BITMAP dimensions
        //
        let mut bmp = BITMAP::default();
        if GetObjectW(
            info.hbmColor.into(),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bmp as *mut _ as *mut _ as *mut std::ffi::c_void), // Use std::ffi::c_void
        ) == 0
        {
            return Err(anyhow::anyhow!("GetObjectW failed"));
        }

        let width = bmp.bmWidth as u32;
        let height = bmp.bmHeight as u32;

        //
        // 3. Prepare BITMAPINFO for GetDIBits
        //
        let mut bi = BITMAPINFO::default();
        bi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bi.bmiHeader.biWidth = width as i32;
        bi.bmiHeader.biHeight = -(height as i32); // top-down
        bi.bmiHeader.biPlanes = 1;
        bi.bmiHeader.biBitCount = 32;
        bi.bmiHeader.biCompression = BI_RGB.0; // Cast to u32

        //
        // 4. Allocate BGRA buffer
        //
        let mut bgra_data = vec![0u8; (width * height * 4) as usize];
        unsafe {
            let hdc = GetDC(None);
            if hdc.0.is_null() {
                panic!("GetDC failed");
            }

            let result = GetDIBits(
                hdc,
                info.hbmColor,
                0,
                height as u32,
                Some(bgra_data.as_mut_ptr() as *mut _),
                &mut bi,
                DIB_RGB_COLORS,
            );

            ReleaseDC(None, hdc);

            if result == 0 {
                panic!("GetDIBits failed");
            }
        }
        //
        // 5. Extract image pixels
        //
        let hdc = GetDC(None);
        let result = GetDIBits(
            hdc,
            info.hbmColor,
            0,
            height,
            Some(bgra_data.as_mut_ptr() as *mut _),
            &mut bi,
            DIB_RGB_COLORS,
        );
        ReleaseDC(None, hdc);
        if result == 0 {
            return Err(anyhow::anyhow!("GetDIBits failed"));
        }

        //
        // 6. BGRA → RGBA
        //
        for px in bgra_data.chunks_exact_mut(4) {
            px.swap(0, 2); // B ↔ R
        }

        //
        // 7. Encode PNG using PngEncoder (image 0.25+)
        //
        let img = RgbaImage::from_raw(width, height, bgra_data)
            .ok_or_else(|| anyhow::anyhow!("Failed to build RGBA image"))?;

        let mut png_bytes = Vec::new();
        {
            let encoder = PngEncoder::new(&mut png_bytes);
            encoder.write_image(
                img.as_raw(),
                img.width(),
                img.height(),
                image::ColorType::Rgba8.into(), // Convert to ExtendedColorType
            )?;
        }

        //
        // 8. Cleanup icon bitmaps
        //
        DeleteObject(info.hbmColor.into());
        DeleteObject(info.hbmMask.into());

        Ok(png_bytes)
    }
}

fn extract_largest_hicon(path: &std::path::Path) -> anyhow::Result<HICON> {
    //
    // Convert path → UTF-16
    //
    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    //
    // First call ExtractIconExW with count = 0 to get total icons
    //
    let total = unsafe {
        ExtractIconExW(
            PCWSTR(wide.as_ptr()),
            0,
            None,
            None,
            0, // count = 0 means "just tell me how many icons exist"
        )
    };

    if total == 0 {
        anyhow::bail!("No icons found in exe");
    }

    //
    // Allocate buffers for all icons
    //
    let mut large_icons = vec![HICON::default(); total as usize];
    let mut small_icons = vec![HICON::default(); total as usize];

    //
    // Actually load the icons
    //
    unsafe {
        ExtractIconExW(
            PCWSTR(wide.as_ptr()),
            0,
            Some(large_icons.as_mut_ptr()),
            Some(small_icons.as_mut_ptr()),
            total,
        );
    }

    //
    // Determine the largest icon
    //
    let mut best: Option<(HICON, i32)> = None;

    let check_icon = |icon: HICON| -> Option<(HICON, i32)> {
        if icon.0.is_null() {
            return None;
        }

        // Get icon bitmap info
        let mut info = windows::Win32::UI::WindowsAndMessaging::ICONINFO::default();
        unsafe {
            if !GetIconInfo(icon, &mut info).is_ok() {
                return None;
            }
        }

        let mut bmp = BITMAP::default();
        let ok = unsafe {
            GetObjectW(
                info.hbmColor.into(),
                std::mem::size_of::<BITMAP>() as i32,
                Some(&mut bmp as *mut _ as *mut _ as *mut std::ffi::c_void),
            )
        };

        unsafe {
            windows::Win32::Graphics::Gdi::DeleteObject(info.hbmColor.into());
            windows::Win32::Graphics::Gdi::DeleteObject(info.hbmMask.into());
        }

        if ok == 0 {
            return None;
        }

        let size = bmp.bmWidth * bmp.bmHeight;
        Some((icon, size))
    };

    // Try large icons first, then fallback to small
    for icon in large_icons.iter().copied().chain(small_icons.iter().copied()) {
        if let Some((handle, score)) = check_icon(icon) {
            match best {
                None => best = Some((handle, score)),
                Some((_, best_score)) => {
                    if score > best_score {
                        best = Some((handle, score));
                    }
                }
            }
        }
    }

    let Some((best_icon, _)) = best else {
        anyhow::bail!("Failed to find usable icon");
    };

    //
    // Cleanup unused icons (except best one)
    //
    for icon_pair in large_icons.into_iter().chain(small_icons.into_iter()) {
        if icon_pair.0.is_null() || icon_pair == best_icon {
            continue;
        }
        unsafe { windows::Win32::UI::WindowsAndMessaging::DestroyIcon(icon_pair) };
    }

    Ok(best_icon)
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
