use base64::Engine;
use serde_json::Value;
use tauri::State;

use crate::AppState;

#[cfg(target_os = "macos")]
fn generate_pdf_thumbnail(path: &std::path::Path) -> Option<Vec<u8>> {
    let tmp_dir = std::env::temp_dir().join("j12_pdf_thumbs");
    let _ = std::fs::create_dir_all(&tmp_dir);
    let output = std::process::Command::new("qlmanage")
        .arg("-t")
        .arg("-s")
        .arg("400")
        .arg("-o")
        .arg(&tmp_dir)
        .arg(path)
        .output()
        .ok()?;
    if output.status.success() {
        let fname = path.file_name()?.to_str()?;
        let thumb_filename = format!("{}.png", fname);
        let thumb_path = tmp_dir.join(&thumb_filename);
        if thumb_path.exists() {
            let data = std::fs::read(&thumb_path).ok();
            let _ = std::fs::remove_file(&thumb_path);
            return data;
        }
    }
    None
}

#[cfg(not(target_os = "macos"))]
fn generate_pdf_thumbnail(_path: &std::path::Path) -> Option<Vec<u8>> {
    None
}

#[tauri::command]
pub async fn get_attachment_preview(
    state: State<'_, AppState>,
    input: Value,
) -> Result<Option<String>, String> {
    let attachment_id = input["attachment_id"].as_str()
        .or_else(|| input["attachmentId"].as_str())
        .or_else(|| input["id"].as_str())
        .or_else(|| input["input"]["attachment_id"].as_str())
        .or_else(|| input["input"]["attachmentId"].as_str())
        .or_else(|| input["input"]["id"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("");

    let direct_path = input["stored_path"].as_str()
        .or_else(|| input["storedPath"].as_str())
        .or_else(|| input["input"]["stored_path"].as_str())
        .or_else(|| input["input"]["storedPath"].as_str())
        .unwrap_or("");

    let (stored_path, email_id, filename, mime_type): (Option<String>, Option<String>, Option<String>, Option<String>) = if !direct_path.is_empty() {
        (Some(direct_path.to_string()), None, None, None)
    } else if !attachment_id.is_empty() {
        let db = state.db.lock().await;
        db.conn.query_row(
            "SELECT stored_path, email_id, filename, mime_type FROM attachments WHERE id=?1",
            [attachment_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        ).ok().unwrap_or((None, None, None, None))
    } else {
        (None, None, None, None)
    };

    if let Some(ref path_str) = stored_path {
        if !path_str.is_empty() {
            let path = std::path::Path::new(path_str);
            if path.exists() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

                #[cfg(target_os = "macos")]
                if ext == "pdf" {
                    if let Some(png_bytes) = generate_pdf_thumbnail(path) {
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
                        return Ok(Some(format!("data:image/png;base64,{}", b64)));
                    }
                }

                if let Ok(data) = std::fs::read(path) {
                    let mime = match ext.as_str() {
                        "jpg" | "jpeg" => "image/jpeg",
                        "png" => "image/png",
                        "gif" => "image/gif",
                        "webp" => "image/webp",
                        "svg" => "image/svg+xml",
                        "bmp" => "image/bmp",
                        "ico" => "image/x-icon",
                        "pdf" => "application/pdf",
                        _ => "application/octet-stream",
                    };
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
                    return Ok(Some(format!("data:{};base64,{}", mime, b64)));
                }
            }
        }
    }

    if let Some(eid) = email_id {
        let db = state.db.lock().await;
        if let Ok((body_text, body_html, headers_raw)) = db.conn.query_row(
            "SELECT body_text, body_html, headers_raw FROM emails WHERE id=?1",
            [&eid],
            |r| Ok((
                r.get::<_, Option<String>>(0)?.unwrap_or_default(),
                r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                r.get::<_, Option<String>>(2)?.unwrap_or_default()
            )),
        ) {
            let full_raw = format!("{}\n{}\n{}", headers_raw, body_text, body_html);
            let fname = filename.unwrap_or_default();
            let lower_fname = fname.to_lowercase();
            
            let mut search_terms = vec![fname.as_str()];
            if !lower_fname.is_empty() {
                search_terms.push(lower_fname.as_str());
            }

            for term in search_terms {
                if let Some(pos) = full_raw.find(term) {
                    let window = &full_raw[pos..full_raw.len().min(pos + 1_500_000)];
                    if let Some(b64_start) = window.find("\r\n\r\n").or_else(|| window.find("\n\n")) {
                        let candidate = &window[b64_start..];
                        let clean_b64: String = candidate.chars()
                            .take_while(|c| c.is_ascii_alphanumeric() || *c == '+' || *c == '/' || *c == '=' || c.is_ascii_whitespace())
                            .filter(|c| !c.is_ascii_whitespace())
                            .collect();
                        if clean_b64.len() > 100 {
                            let mime = mime_type.clone().unwrap_or_else(|| {
                                if lower_fname.ends_with(".pdf") {
                                    "application/pdf".to_string()
                                } else {
                                    "image/png".to_string()
                                }
                            });
                            return Ok(Some(format!("data:{};base64,{}", mime, clean_b64)));
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}
