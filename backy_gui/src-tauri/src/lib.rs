// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri_plugin_dialog::DialogExt;
use backy_core::{backup_start, chunk_file, init_repo, save_blob, list_blobs, save_blob_local};

#[tauri::command]
fn backup_start_cmd(source: String) -> Result<String, String> {
  backup_start(&source).map_err(|e| e.to_string())
}

#[tauri::command]
fn chunk_file_cmd(path: String) -> Result<usize, String> {
  chunk_file(&path)
    .map(|chunks| chunks.len())
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn init_repo_cmd() -> Result<String, String> {
  init_repo()
    .map(|(data_dir, _)| data_dir.to_string_lossy().into_owned())
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_blob_cmd(blob: Vec<u8>) -> Result<String, String> {
  save_blob(&blob)
    .map(|id| id.to_string())
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_blobs_cmd() -> Result<Vec<String>, String> {
  list_blobs()
    .map(|ids| ids.into_iter().map(|id| id.to_string()).collect())
    .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]

#[tauri::command]
fn save_blob_local_cmd(path: String, dest_dir: String) -> Result<String, String> {
  let metadata = std::fs::metadata(&path).map_err(|e| e.to_string())?;
  
  if metadata.is_file() {
    // Handle file
    let data = std::fs::read(&path).map_err(|e| e.to_string())?;
    let filename = std::path::Path::new(&path)
      .file_name()
      .and_then(|n| n.to_str())
      .unwrap_or("file")
      .to_string();
    save_blob_local(&data, &dest_dir, &filename).map_err(|e| e.to_string())
  } else if metadata.is_dir() {
    // Handle directory
    let dir_name = std::path::Path::new(&path)
      .file_name()
      .and_then(|n| n.to_str())
      .unwrap_or("directory")
      .to_string();
    let dest_path = std::path::Path::new(&dest_dir).join(&dir_name);
    std::fs::create_dir_all(&dest_path).map_err(|e| e.to_string())?;
    
    // Copy all files recursively
    for entry in std::fs::read_dir(&path).map_err(|e| e.to_string())? {
      let entry = entry.map_err(|e| e.to_string())?;
      let entry_path = entry.path();
      if entry_path.is_file() {
        let data = std::fs::read(&entry_path).map_err(|e| e.to_string())?;
        let filename = entry_path
          .file_name()
          .and_then(|n| n.to_str())
          .unwrap_or("file")
          .to_string();
        save_blob_local(&data, &dest_path.to_string_lossy(), &filename)
          .map_err(|e| e.to_string())?;
      }
    }
    Ok(format!("Dossier sauvegardé dans : {}", dest_path.to_string_lossy()))
  } else {
    Err("Le chemin spécifié n'est ni un fichier ni un dossier".to_string())
  }
}

#[tauri::command]
async fn open_file_dialog(app: tauri::AppHandle) -> Option<String> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog().file().pick_file(move |path| {
        tx.send(path.map(|p| p.to_string())).unwrap();
    });
    rx.recv().unwrap()
}

#[tauri::command]
async fn open_directory_dialog(app: tauri::AppHandle) -> Option<String> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog().file().pick_folder(move |path| {
        tx.send(path.map(|p| p.to_string())).unwrap();
    });
    rx.recv().unwrap()
}
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      backup_start_cmd,
      chunk_file_cmd,
      init_repo_cmd,
      save_blob_cmd,
      list_blobs_cmd,
      save_blob_local_cmd,
      open_file_dialog,
      open_directory_dialog
    ])
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_log::Builder::default()
      .level(log::LevelFilter::Info)
      .build())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
