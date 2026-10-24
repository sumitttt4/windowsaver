#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use base64::Engine as _;
use notify::{RecursiveMode, Watcher};
use pile_core::{Card, Engine, Native, Snapshot};
use serde::Serialize;
use std::{
    io::Cursor,
    path::Path,
    sync::{Arc, Mutex},
};
use tauri::{Emitter, Manager};
static PREVIEW_GATE: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);
static PREVIEW_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
struct Desktop {
    engine: Arc<Mutex<Engine>>,
    watcher: Mutex<Option<notify::RecommendedWatcher>>,
}
fn access<T>(
    state: &Desktop,
    f: impl FnOnce(&mut Engine) -> Result<T, String>,
) -> Result<T, String> {
    let mut e = state
        .engine
        .lock()
        .map_err(|_| "File engine unavailable. Restart the app.")?;
    f(&mut e)
}
fn watch(app: &tauri::AppHandle, state: &Desktop) -> Result<(), String> {
    let handle = app.clone();
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
        let _ = handle.emit("pile-changed", event.is_ok());
    })
    .map_err(|e| e.to_string())?;
    access(state, |e| {
        for path in [&e.config.downloads, &e.config.screenshots]
            .into_iter()
            .flatten()
        {
            watcher
                .watch(path, RecursiveMode::NonRecursive)
                .map_err(|x| x.to_string())?;
        }
        Ok(())
    })?;
    *state.watcher.lock().map_err(|_| "Watcher unavailable")? = Some(watcher);
    Ok(())
}
#[tauri::command]
async fn snapshot(state: tauri::State<'_, Desktop>) -> Result<Snapshot, String> {
    let engine = state.engine.clone();
    tauri::async_runtime::spawn_blocking(move || {
        engine
            .lock()
            .map_err(|_| "Engine unavailable".to_string())?
            .snapshot()
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn pick_folder(app: tauri::AppHandle, kind: String) -> Result<Option<Snapshot>, String> {
    if !["downloads", "screenshots", "keep"].contains(&kind.as_str()) {
        return Err("Unknown folder".into());
    }
    let initial = access(&app.state::<Desktop>(), |e| {
        Ok(match kind.as_str() {
            "downloads" => e.config.downloads.clone(),
            "screenshots" => e.config.screenshots.clone(),
            _ => e.config.keep.clone(),
        })
    })?
    .or_else(|| native_folder(&kind));
    let folder = tauri::async_runtime::spawn_blocking(move || {
        let mut dialog =
            rfd::FileDialog::new().set_title("Choose a local folder for Feed the Pile");
        if let Some(path) = initial.filter(|p| p.is_dir()) {
            dialog = dialog.set_directory(path);
        }
        dialog.pick_folder()
    })
    .await
    .map_err(|e| e.to_string())?;
    let Some(folder) = folder else {
        return Ok(None);
    };
    let state = app.state::<Desktop>();
    access(&state, |e| e.configure(&kind, &folder))?;
    watch(&app, &state)?;
    access(&state, |e| e.snapshot()).map(Some)
}
#[tauri::command]
async fn decide(
    state: tauri::State<'_, Desktop>,
    pile: String,
    path: String,
    id: String,
    kind: String,
) -> Result<Snapshot, String> {
    #[cfg(not(windows))]
    {
        let _ = (state, pile, path, id, kind);
        return Err("File decisions are only enabled on Windows.".into());
    }
    #[cfg(windows)]
    {
        let engine = state.engine.clone();
        tauri::async_runtime::spawn_blocking(move || {
            let mut e = engine
                .lock()
                .map_err(|_| "Engine unavailable".to_string())?;
            e.decide(&pile, Path::new(&path), &id, &kind)?;
            e.snapshot()
        })
        .await
        .map_err(|e| e.to_string())?
    }
}
#[tauri::command]
async fn undo(state: tauri::State<'_, Desktop>) -> Result<Snapshot, String> {
    let engine = state.engine.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut e = engine
            .lock()
            .map_err(|_| "Engine unavailable".to_string())?;
        e.undo()?;
        e.snapshot()
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn recover(state: tauri::State<'_, Desktop>) -> Result<Snapshot, String> {
    let engine = state.engine.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut e = engine
            .lock()
            .map_err(|_| "Engine unavailable".to_string())?;
        e.recover()?;
        e.snapshot()
    })
    .await
    .map_err(|e| e.to_string())?
}
fn approved(state: &Desktop, pile: &str, path: &str, id: &str) -> Result<Card, String> {
    access(state, |e| e.checked_card(pile, Path::new(path), id))
}
#[derive(Serialize)]
struct Preview {
    kind: String,
    data: String,
}
fn decoded(path: &Path) -> Result<image::DynamicImage, String> {
    let reader = image::ImageReader::open(path)
        .map_err(|e| e.to_string())?
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(12000);
    limits.max_image_height = Some(12000);
    limits.max_alloc = Some(128 * 1024 * 1024);
    let mut reader = reader;
    reader.limits(limits);
    reader.decode().map_err(|e| e.to_string())
}
#[tauri::command]
async fn preview(
    state: tauri::State<'_, Desktop>,
    pile: String,
    path: String,
    id: String,
) -> Result<Preview, String> {
    let file = approved(&state, &pile, &path, &id)?;
    use std::sync::atomic::Ordering;
    let revision = PREVIEW_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let permit = PREVIEW_GATE.acquire().await.map_err(|e| e.to_string())?;
    if PREVIEW_GENERATION.load(Ordering::SeqCst) != revision {
        return Err("Preview superseded by a newer card".into());
    }

    tauri::async_runtime::spawn_blocking(move || {
        let _permit = permit;
        if file.extension == "pdf" {
            if file.size > 30 * 1024 * 1024 {
                return Err("PDF exceeds the 30 MB preview limit. Open it with Space.".into());
            }
            let bytes = std::fs::read(&file.path).map_err(|e| e.to_string())?;
            return Ok(Preview {
                kind: "pdf".into(),
                data: base64::engine::general_purpose::STANDARD.encode(bytes),
            });
        }
        if !["png", "jpg", "jpeg", "webp", "gif", "bmp"].contains(&file.extension.as_str()) {
            return Err("Use Space to open this file.".into());
        }
        if file.size > 40 * 1024 * 1024 {
            return Err("Image exceeds the 40 MB preview limit. Open it with Space.".into());
        }
        let image = decoded(&file.path)?.thumbnail(720, 520);
        let mut bytes = Cursor::new(Vec::new());
        image
            .write_to(&mut bytes, image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        Ok(Preview {
            kind: "image".into(),
            data: base64::engine::general_purpose::STANDARD.encode(bytes.into_inner()),
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn copy_image(
    state: tauri::State<'_, Desktop>,
    pile: String,
    path: String,
    id: String,
) -> Result<(), String> {
    if pile != "screenshots" {
        return Err("Copy is available in Screenshots.".into());
    }
    let file = approved(&state, &pile, &path, &id)?;
    let permit = PREVIEW_GATE.acquire().await.map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn_blocking(move || {
        let _permit = permit;
        #[cfg(windows)]
        {
            use std::borrow::Cow;
            if file.size > 40 * 1024 * 1024 {
                return Err("Image exceeds the clipboard size limit".into());
            }
            let image = decoded(&file.path)?.into_rgba8();
            let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
            clipboard
                .set_image(arboard::ImageData {
                    width: image.width() as usize,
                    height: image.height() as usize,
                    bytes: Cow::Owned(image.into_raw()),
                })
                .map_err(|e| e.to_string())
        }
        #[cfg(not(windows))]
        {
            let _ = file;
            Err("Clipboard copying requires Windows".into())
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn open_file(
    state: tauri::State<'_, Desktop>,
    pile: String,
    path: String,
    id: String,
) -> Result<(), String> {
    let mut file = approved(&state, &pile, &path, &id)?;
    #[cfg(windows)]
    {
        let original = file.path.to_string_lossy();
        file.path = std::path::PathBuf::from(original.strip_prefix(r"\\?\").unwrap_or(&original));
    }
    tauri::async_runtime::spawn_blocking(move || {
        #[cfg(windows)]
        {
            if [
                "exe", "com", "bat", "cmd", "ps1", "msi", "scr", "vbs", "js", "lnk", "url",
            ]
            .contains(&file.extension.as_str())
            {
                return std::process::Command::new("explorer.exe")
                    .arg(format!("/select,{}", file.path.display()))
                    .spawn()
                    .map(|_| ())
                    .map_err(|e| e.to_string());
            }
        }
        open::that(file.path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
fn native_folder(kind: &str) -> Option<std::path::PathBuf> {
    #[cfg(windows)]
    {
        use windows::Win32::{System::Com::CoTaskMemFree, UI::Shell::*};
        let id = match kind {
            "downloads" => FOLDERID_Downloads,
            "screenshots" => FOLDERID_Screenshots,
            "pictures" => FOLDERID_Pictures,
            _ => FOLDERID_Documents,
        };
        unsafe {
            let ptr = SHGetKnownFolderPath(&id, KF_FLAG_DONT_VERIFY, None).ok()?;
            let path = ptr.to_string().ok().map(std::path::PathBuf::from);
            CoTaskMemFree(Some(ptr.0 as *const std::ffi::c_void));
            path
        }
    }
    #[cfg(not(windows))]
    {
        let _ = kind;
        None
    }
}
#[tauri::command]
async fn folder_suggestions() -> Result<serde_json::Value, String> {
    let downloads = native_folder("downloads");
    let screenshots = native_folder("screenshots");
    let fallback = native_folder("pictures").map(|p| p.join("Screenshots"));
    Ok(
        serde_json::json!({"downloads":downloads,"screenshotsKnownFolder":screenshots,"picturesFallback":fallback,"status":"Suggestions only. Confirm where a screenshot made with your usual shortcut actually lands."}),
    )
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            let dir = app.path().app_local_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let engine =
                Engine::open(&dir.join("pile.sqlite3"), Native).map_err(std::io::Error::other)?;
            app.manage(Desktop {
                engine: Arc::new(Mutex::new(engine)),
                watcher: Mutex::new(None),
            });
            if let Err(error) = watch(app.handle(), &app.state::<Desktop>()) {
                eprintln!("Folder watcher unavailable: {error}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            snapshot,
            pick_folder,
            decide,
            undo,
            recover,
            preview,
            copy_image,
            open_file,
            folder_suggestions
        ])
        .run(tauri::generate_context!())
        .expect("Unable to start Feed the Pile");
}
