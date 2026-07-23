#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod recovery;

use recovery::*;
use tauri::State;
use std::sync::Mutex;

struct AppState {
    scan_results: Mutex<Vec<RecoveredFile>>,
    is_scanning: Mutex<bool>,
    config: Mutex<Option<ScanConfig>>,
}

#[tauri::command]
fn get_available_drives() -> Vec<DriveInfo> {
    get_drives()
}

#[tauri::command]
fn start_scan(config: ScanConfig, state: State<AppState>) -> Result<Vec<RecoveredFile>, String> {
    let mut scanning = state.is_scanning.lock().map_err(|e| e.to_string())?;
    if *scanning {
        return Err("Scan already in progress".to_string());
    }
    *scanning = true;

    // Store config
    if let Ok(mut cfg) = state.config.lock() {
        *cfg = Some(config.clone());
    }

    let results = match config.scan_type.as_str() {
        "fast" => fast_scan(&config),
        "deep" => deep_scan(&config),
        _ => fast_scan(&config),
    };

    // Store results
    if let Ok(mut stored) = state.scan_results.lock() {
        *stored = results.clone();
    }

    *scanning = false;
    Ok(results)
}

#[tauri::command]
fn recover_selected(file_ids: Vec<String>, state: State<AppState>) -> Result<Vec<RecoveredFile>, String> {
    let stored = state.scan_results.lock().map_err(|e| e.to_string())?;
    let config = state.config.lock().map_err(|e| e.to_string())?;

    let config = config.as_ref().ok_or("No scan configuration found")?;

    let files_to_recover: Vec<RecoveredFile> = stored
        .iter()
        .filter(|f| file_ids.contains(&f.id))
        .cloned()
        .collect();

    let results = recover_files(&files_to_recover, config);
    Ok(results)
}

#[tauri::command]
fn recover_all(state: State<AppState>) -> Result<Vec<RecoveredFile>, String> {
    let stored = state.scan_results.lock().map_err(|e| e.to_string())?;
    let config = state.config.lock().map_err(|e| e.to_string())?;

    let config = config.as_ref().ok_or("No scan configuration found")?;

    let results = recover_files(&stored, config);
    Ok(results)
}

#[tauri::command]
fn stop_scan(state: State<AppState>) -> Result<(), String> {
    let mut scanning = state.is_scanning.lock().map_err(|e| e.to_string())?;
    *scanning = false;
    Ok(())
}

#[tauri::command]
fn get_scan_status(state: State<AppState>) -> Result<ScanProgress, String> {
    let scanning = state.is_scanning.lock().map_err(|e| e.to_string())?;
    let stored = state.scan_results.lock().map_err(|e| e.to_string())?;

    let status = if *scanning {
        "scanning".to_string()
    } else if stored.is_empty() {
        "idle".to_string()
    } else {
        "complete".to_string()
    };

    Ok(ScanProgress {
        phase: status.clone(),
        current_file: String::new(),
        files_found: stored.len() as u64,
        files_recovered: stored.iter().filter(|f| f.status.contains("recovered")).count() as u64,
        total_size_recovered: stored.iter().filter(|f| f.status.contains("recovered")).map(|f| f.size).sum(),
        progress_percent: if *scanning { 50.0 } else if stored.is_empty() { 0.0 } else { 100.0 },
        elapsed_seconds: 0,
        estimated_remaining: 0,
        scan_speed_mbps: 0.0,
        status,
    })
}

#[tauri::command]
fn filter_files(
    state: State<AppState>,
    category: Option<String>,
    min_size: Option<u64>,
    max_size: Option<u64>,
    exclude_damaged: Option<bool>,
    exclude_thumbnails: Option<bool>,
) -> Result<Vec<RecoveredFile>, String> {
    let stored = state.scan_results.lock().map_err(|e| e.to_string())?;

    let filtered: Vec<RecoveredFile> = stored
        .iter()
        .filter(|f| {
            if let Some(ref cat) = category {
                if !cat.is_empty() && &f.category != cat {
                    return false;
                }
            }
            if let Some(min) = min_size {
                if f.size < min {
                    return false;
                }
            }
            if let Some(max) = max_size {
                if f.size > max {
                    return false;
                }
            }
            if exclude_damaged.unwrap_or(false) && f.is_damaged {
                return false;
            }
            if exclude_thumbnails.unwrap_or(false) && f.is_thumbnail {
                return false;
            }
            true
        })
        .cloned()
        .collect();

    Ok(filtered)
}

#[tauri::command]
fn select_folder(_app: tauri::AppHandle) -> Result<String, String> {
    // Use tauri dialog
    let result = tauri::api::dialog::blocking::FileDialogBuilder::new()
        .set_title("Select Recovery Destination")
        .pick_folder();

    match result {
        Some(path) => Ok(path.to_string_lossy().to_string()),
        None => Err("No folder selected".to_string()),
    }
}

#[tauri::command]
fn get_file_categories() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({"id": "all", "name": "Todos os Arquivos", "icon": "📁", "extensions": ["*"]}),
        serde_json::json!({"id": "images", "name": "Imagens", "icon": "🖼️", "extensions": ["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "cr2", "heic"]}),
        serde_json::json!({"id": "videos", "name": "Vídeos", "icon": "🎬", "extensions": ["mp4", "avi", "mkv", "mov", "flv", "wmv", "webm"]}),
        serde_json::json!({"id": "audio", "name": "Áudios", "icon": "🎵", "extensions": ["mp3", "wav", "flac", "ogg", "aac", "m4a", "wma"]}),
        serde_json::json!({"id": "documents", "name": "Documentos", "icon": "📄", "extensions": ["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "rtf"]}),
        serde_json::json!({"id": "archives", "name": "Arquivos Compactados", "icon": "📦", "extensions": ["zip", "rar", "7z", "tar", "gz"]}),
        serde_json::json!({"id": "emails", "name": "Emails", "icon": "📧", "extensions": ["eml", "msg", "pst"]}),
        serde_json::json!({"id": "databases", "name": "Bancos de Dados", "icon": "🗄️", "extensions": ["db", "sql", "sqlite", "mdb"]}),
    ]
}

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .manage(AppState {
            scan_results: Mutex::new(Vec::new()),
            is_scanning: Mutex::new(false),
            config: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            get_available_drives,
            start_scan,
            recover_selected,
            recover_all,
            stop_scan,
            get_scan_status,
            filter_files,
            select_folder,
            get_file_categories,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
