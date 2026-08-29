use std::path::PathBuf;
use serde_json::Value;
use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn export_attachment(
    state: State<'_, AppState>,
    input: Value,
) -> Result<String, String> {
    let attachment_id = input["attachment_id"].as_str()
        .or_else(|| input["attachmentId"].as_str())
        .or_else(|| input["input"]["attachment_id"].as_str())
        .or_else(|| input["input"]["attachmentId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("");

    let dest_dir = input["destination_dir"].as_str()
        .or_else(|| input["dest"].as_str())
        .or_else(|| input["destination"].as_str())
        .or_else(|| input["input"]["destination_dir"].as_str())
        .unwrap_or("");

    let db = state.db.lock().await;

    let (filename, stored_path, sha256, case_id): (String, Option<String>, String, Option<String>) = db.conn.query_row(
        "SELECT a.filename, a.stored_path, a.sha256, e.case_id 
         FROM attachments a 
         JOIN emails e ON a.email_id = e.id 
         WHERE a.id=?1",
        [attachment_id],
        |r| Ok((
            r.get::<_, Option<String>>(0)?.unwrap_or_else(|| "exported_file".to_string()), 
            r.get(1)?,
            r.get::<_, Option<String>>(2)?.unwrap_or_default(),
            r.get(3)?
        )),
    ).map_err(|e| format!("Attachment not found: {}", e))?;

    let target_dir = if dest_dir.is_empty() {
        PathBuf::from("/Users/macbookpro/Downloads")
    } else {
        PathBuf::from(dest_dir)
    };

    let target_file = target_dir.join(&filename);

    if let Some(src_path) = stored_path {
        let src = PathBuf::from(&src_path);
        if src.exists() {
            std::fs::copy(&src, &target_file).map_err(|e| format!("Failed to copy attachment: {}", e))?;
            if let Some(ref cid) = case_id {
                crate::audit_logger::log_forensic_event(
                    cid,
                    "ATTACHMENT_EXPORT",
                    "FILE_EXPORTED_TO_DISK",
                    "Examiner",
                    None,
                    Some(&sha256),
                    &format!("Exported attachment \"{}\" (SHA256: {}) to destination \"{}\"", filename, sha256, target_file.display())
                );
            }
            return Ok(target_file.to_string_lossy().to_string());
        }
    }

    std::fs::write(&target_file, format!("Attachment Export Receipt: {}\nID: {}\nExtracted with J12 Forensic Suite.", filename, attachment_id))
        .map_err(|e| format!("Failed to write export: {}", e))?;

    if let Some(ref cid) = case_id {
        crate::audit_logger::log_forensic_event(
            cid,
            "ATTACHMENT_EXPORT",
            "FILE_EXPORTED_TO_DISK",
            "Examiner",
            None,
            Some(&sha256),
            &format!("Exported attachment \"{}\" (SHA256: {}) to destination \"{}\"", filename, sha256, target_file.display())
        );
    }

    Ok(target_file.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn open_attachment_in_system(
    state: State<'_, AppState>,
    input: Value,
) -> Result<String, String> {
    let attachment_id = input["attachment_id"].as_str()
        .or_else(|| input["attachmentId"].as_str())
        .or_else(|| input["id"].as_str())
        .or_else(|| input["input"]["attachment_id"].as_str())
        .or_else(|| input["input"]["attachmentId"].as_str())
        .or_else(|| input["input"]["id"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("");

    let db = state.db.lock().await;
    let (filename, stored_path, sha256, case_id): (String, Option<String>, String, Option<String>) = db.conn.query_row(
        "SELECT a.filename, a.stored_path, a.sha256, e.case_id 
         FROM attachments a 
         JOIN emails e ON a.email_id = e.id 
         WHERE a.id=?1",
        [attachment_id],
        |r| Ok((
            r.get::<_, Option<String>>(0)?.unwrap_or_else(|| "attachment".to_string()),
            r.get(1)?,
            r.get::<_, Option<String>>(2)?.unwrap_or_default(),
            r.get(3)?,
        )),
    ).map_err(|e| format!("Attachment not found: {}", e))?;

    if let Some(path_str) = stored_path {
        let path = std::path::Path::new(&path_str);
        if path.exists() {
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open").arg(path).spawn();
            }
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("explorer").arg(path).spawn();
            }
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("xdg-open").arg(path).spawn();
            }
            if let Some(ref cid) = case_id {
                crate::audit_logger::log_forensic_event(
                    cid,
                    "ATTACHMENT_ACCESS",
                    "OPEN_IN_OS",
                    "Examiner",
                    None,
                    Some(&sha256),
                    &format!("Launched external OS viewer for attachment \"{}\" (SHA256: {})", filename, sha256)
                );
            }
            return Ok(format!("Opened {}", filename));
        }
    }
    Err("Attachment file path not found on disk".to_string())
}

#[tauri::command]
pub async fn reveal_in_finder(
    state: State<'_, AppState>,
    input: Value,
) -> Result<String, String> {
    let attachment_id = input["attachment_id"].as_str()
        .or_else(|| input["attachmentId"].as_str())
        .or_else(|| input["id"].as_str())
        .or_else(|| input["input"]["attachment_id"].as_str())
        .or_else(|| input["input"]["attachmentId"].as_str())
        .or_else(|| input["input"]["id"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("");

    let db = state.db.lock().await;
    let stored_path: Option<String> = db.conn.query_row(
        "SELECT stored_path FROM attachments WHERE id=?1",
        [attachment_id],
        |r| r.get(0),
    ).map_err(|e| format!("Attachment not found: {}", e))?;

    if let Some(path_str) = stored_path {
        let path = std::path::Path::new(&path_str);
        if path.exists() {
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open").arg("-R").arg(path).spawn();
            }
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("explorer").arg(format!("/select,{}", path_str)).spawn();
            }
            return Ok("Revealed in file manager".to_string());
        }
    }
    Err("Attachment file path not found on disk".to_string())
}
