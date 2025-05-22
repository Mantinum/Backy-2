// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri_plugin_dialog::DialogExt;
use backy_core::{backup_start, chunk_file, init_repo, save_blob, list_blobs, save_blob_local};
use std::path::Path; // Removed PathBuf as it's unused
use serde::Deserialize;
use log::{info, error}; // Added for logging

mod sftp;
use sftp::SftpClient;

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
    save_blob_local(&data, &dest_dir, &filename).map_err(|e| e.to_string())?;
    Ok("Fichier sauvegardé avec succès".to_string())
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SftpBackupArgs {
  host: String,
  port: u16,
  username: String,
  password: String,
  local_path: String,
  remote_path: String,
}

#[tauri::command]
fn sftp_backup(args: SftpBackupArgs) -> Result<String, String> {
  info!("SFTP Backup: Attempting SFTP connection to {}:{}", args.host, args.port);
  
  let client = SftpClient::new(&args.host, args.port, &args.username, &args.password)
      .map_err(|e| {
          error!("SFTP Backup: Connection failed: {}", e);
          e.to_string()
      })?;
  
  info!("SFTP Backup: Ensuring remote directory exists: {}", args.remote_path);
  client.create_directory(&args.remote_path)
      .map_err(|e| {
          error!("SFTP Backup: Failed to create remote directory '{}': {}", args.remote_path, e);
          e.to_string()
      })?;
  
  let local_path = Path::new(&args.local_path);
  let file_name = local_path.file_name().ok_or_else(|| {
      let err_msg = format!("SFTP Backup: Invalid local path, could not extract filename: {}", args.local_path);
      error!("{}", err_msg);
      err_msg
  })?.to_str().ok_or_else(|| {
      let err_msg = format!("SFTP Backup: Filename from local path is not valid UTF-8: {}", args.local_path);
      error!("{}", err_msg);
      err_msg
  })?;

  let actual_remote_target_path = Path::new(&args.remote_path).join(file_name);
  let actual_remote_target_path_str = actual_remote_target_path.to_str().ok_or_else(|| {
    let err_msg = format!("SFTP Backup: Constructed remote path is not valid UTF-8: {}", actual_remote_target_path.display());
    error!("{}", err_msg);
    err_msg
  })?;

  info!("SFTP Backup: Uploading file {} to {}", args.local_path, actual_remote_target_path_str);
  client.upload_file(local_path, actual_remote_target_path_str)
      .map_err(|e| {
          error!("SFTP Backup: File upload failed for '{}' to '{}': {}", args.local_path, actual_remote_target_path_str, e);
          e.to_string()
      })?;
  
  info!("SFTP Backup: Backup successful for {}", args.local_path);
  Ok(format!("File '{}' backed up successfully to '{}'", file_name, args.remote_path))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SftpListDirectoryArgs {
    host: String,
    port: u16,
    username: String,
    password: String,
    remote_path: String,
}

#[tauri::command]
fn sftp_list_directory(args: SftpListDirectoryArgs) -> Result<Vec<String>, String> {
    info!("SFTP List Directory: Attempting connection to {}:{} for path '{}'", args.host, args.port, args.remote_path);
    let client = SftpClient::new(&args.host, args.port, &args.username, &args.password)
        .map_err(|e| {
            error!("SFTP List Directory: Connection failed for {}:{}: {}", args.host, args.port, e);
            e.to_string()
        })?;
    
    let entries = client.list_directory(&args.remote_path)
        .map_err(|e| {
            error!("SFTP List Directory: Failed to list directory '{}': {}", args.remote_path, e);
            e.to_string()
        })?;
    
    info!("SFTP List Directory: Successfully listed directory '{}'", args.remote_path);
    Ok(entries)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SftpDownloadFileArgs {
    host: String,
    port: u16,
    username: String,
    password: String,
    remote_path: String,
    local_path: String,
}

#[tauri::command]
fn sftp_download_file(args: SftpDownloadFileArgs) -> Result<String, String> {
    info!("SFTP Download File: Attempting to download '{}' from {}:{} to '{}'", args.remote_path, args.host, args.port, args.local_path);
    let client = SftpClient::new(&args.host, args.port, &args.username, &args.password)
        .map_err(|e| {
            error!("SFTP Download File: Connection failed for {}:{}: {}", args.host, args.port, e);
            e.to_string()
        })?;

    client.download_file(&args.remote_path, Path::new(&args.local_path))
        .map_err(|e| {
            error!("SFTP Download File: Failed to download '{}' to '{}': {}", args.remote_path, args.local_path, e);
            e.to_string()
        })?;

    info!("SFTP Download File: Successfully downloaded '{}' to '{}'", args.remote_path, args.local_path);
    Ok("File downloaded successfully".to_string())
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
      open_directory_dialog,
      sftp_backup,
      sftp_list_directory,
      sftp_download_file
    ])
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_log::Builder::default()
      .level(log::LevelFilter::Info) // Ensure log level is set
      .build())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
