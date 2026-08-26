use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use crate::AppState;
use crate::models::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CaseAttachmentItem {
    pub id: String,
    pub email_id: String,
    pub filename: String,
    pub sha256: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub stored_path: Option<String>,
    pub entropy: Option<f64>,
    pub risk_flags: Option<String>,
    pub email_subject: Option<String>,
    pub email_from: String,
    pub email_date: Option<String>,
    pub email_risk_score: u8,
    pub category: String,
}

pub fn classify_attachment_category(filename: &str, mime: &str, _entropy: Option<f64>, risk_flags: Option<&str>) -> String {
    let lower_name = filename.to_lowercase();
    let lower_mime = mime.to_lowercase();
    let has_exec_risk = risk_flags.map(|r| r.contains("executable") || r.contains("macro")).unwrap_or(false);

    // ── DANGEROUS / EXECUTABLE ────────────────────────────────────────────────
    let is_dangerous = has_exec_risk
        || matches_any_ext(&lower_name, &[
            ".exe", ".scr", ".bat", ".cmd", ".com", ".pif", ".cpl", ".msi", ".dll", ".sys",
            ".vbs", ".vbe", ".js", ".jse", ".wsf", ".wsh", ".hta",
            ".ps1", ".ps2", ".psm1", ".psd1",
            ".reg", ".lnk", ".url",
            ".docm", ".xlsm", ".pptm", ".xlsb", ".accdb", ".mdb",
            ".jar", ".class",
            ".sh", ".bash", ".zsh", ".fish", ".command",
            ".py",  // as attachment (not source) can be dangerous
            ".rb", ".pl", ".php",
            ".iso", ".img", ".vhd", ".vmdk",
        ])
        || lower_mime.contains("application/x-msdownload")
        || lower_mime.contains("application/x-executable")
        || lower_mime.contains("application/x-dosexec");
    if is_dangerous { return "dangerous".to_string(); }

    // ── IMAGES ────────────────────────────────────────────────────────────────
    let is_image = lower_mime.starts_with("image/")
        || matches_any_ext(&lower_name, &[
            ".jpg", ".jpeg", ".png", ".gif", ".bmp", ".webp",
            ".tif", ".tiff", ".heic", ".heif",
            ".svg", ".ico", ".cur",
            ".raw", ".cr2", ".cr3", ".nef", ".nrw", ".arw", ".orf", ".rw2", ".dng",
            ".psd", ".psb", ".ai", ".eps",
            ".avif", ".jxl", ".apng",
        ]);
    if is_image { return "images".to_string(); }

    // ── ARCHIVES ─────────────────────────────────────────────────────────────
    let is_archive = lower_mime.contains("zip")
        || lower_mime.contains("x-tar")
        || lower_mime.contains("x-rar")
        || lower_mime.contains("x-7z")
        || lower_mime.contains("x-gzip")
        || lower_mime.contains("x-bzip")
        || matches_any_ext(&lower_name, &[
            ".zip", ".rar", ".7z", ".tar", ".gz", ".tgz", ".bz2", ".tbz2",
            ".xz", ".txz", ".lz", ".lzma", ".zst",
            ".cab", ".arj", ".ace", ".lha", ".lzh",
            ".iso", ".dmg", ".pkg", ".deb", ".rpm",
            ".war", ".ear", ".apk", ".ipa", ".appx",
        ]);
    if is_archive { return "archives".to_string(); }

    // ── MEDIA (audio / video) ─────────────────────────────────────────────────
    let is_media = lower_mime.starts_with("audio/")
        || lower_mime.starts_with("video/")
        || matches_any_ext(&lower_name, &[
            ".mp3", ".wav", ".flac", ".aac", ".ogg", ".oga", ".m4a", ".wma", ".aiff", ".aif",
            ".opus", ".mid", ".midi", ".amr", ".ra", ".rm",
            ".mp4", ".avi", ".mov", ".mkv", ".wmv", ".flv", ".webm",
            ".m4v", ".3gp", ".3g2", ".ts", ".mts", ".m2ts", ".vob",
            ".ogv", ".f4v", ".asf",
        ]);
    if is_media { return "media".to_string(); }

    // ── DOCUMENTS ────────────────────────────────────────────────────────────
    let is_document = lower_mime.contains("pdf")
        || lower_mime.contains("officedocument")
        || lower_mime.contains("msword")
        || lower_mime.contains("ms-excel")
        || lower_mime.contains("ms-powerpoint")
        || lower_mime.contains("opendocument")
        || lower_mime.starts_with("text/")
        || matches_any_ext(&lower_name, &[
            // PDF
            ".pdf",
            // Microsoft Office
            ".doc", ".docx", ".dot", ".dotx", ".dotm",
            ".xls", ".xlsx", ".xlt", ".xltx", ".xlam",
            ".ppt", ".pptx", ".pot", ".potx", ".ppsx", ".pps",
            // OpenDocument
            ".odt", ".ods", ".odp", ".odg", ".odf",
            // Apple iWork
            ".pages", ".numbers", ".key",
            // Text / markup
            ".txt", ".rtf", ".md", ".markdown", ".rst",
            ".csv", ".tsv",
            ".html", ".htm", ".xhtml", ".mhtml", ".mht",
            ".xml", ".xsl", ".xslt",
            ".json", ".yaml", ".yml", ".toml", ".ini", ".cfg", ".conf",
            ".log", ".nfo", ".diz",
            // eBook
            ".epub", ".mobi", ".azw", ".azw3",
            // Email (as attachment)
            ".eml", ".msg", ".mbox",
        ]);
    if is_document { return "documents".to_string(); }

    // Default fallback — still show as documents rather than silent "other"
    "documents".to_string()
}

/// Check if filename ends with any of the given extensions
fn matches_any_ext(lower_name: &str, exts: &[&str]) -> bool {
    exts.iter().any(|e| lower_name.ends_with(e))
}

/// Classify using magic bytes when file data is available (more reliable than extension)
pub fn classify_by_magic(data: &[u8], filename: &str, mime: &str) -> String {
    if data.len() >= 4 {
        // JPEG: FF D8 FF
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) { return "images".to_string(); }
        // PNG: 89 50 4E 47
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) { return "images".to_string(); }
        // GIF: GIF8
        if data.starts_with(b"GIF8") { return "images".to_string(); }
        // BMP: BM
        if data.starts_with(b"BM") { return "images".to_string(); }
        // WEBP: RIFF....WEBP
        if data.starts_with(b"RIFF") && data.len() >= 12 && &data[8..12] == b"WEBP" { return "images".to_string(); }
        // PDF: %PDF-
        if data.starts_with(b"%PDF-") { return "documents".to_string(); }
        // ZIP / Office Open XML: PK\x03\x04
        if data.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
            // Office Open XML is a ZIP — check filename for specifics
            let ln = filename.to_lowercase();
            if ln.ends_with(".docx") || ln.ends_with(".xlsx") || ln.ends_with(".pptx") {
                return "documents".to_string();
            }
            return "archives".to_string();
        }
        // RAR: Rar!
        if data.starts_with(b"Rar!") { return "archives".to_string(); }
        // 7z: 37 7A BC AF
        if data.starts_with(&[0x37, 0x7A, 0xBC, 0xAF]) { return "archives".to_string(); }
        // gzip: 1F 8B
        if data.starts_with(&[0x1F, 0x8B]) { return "archives".to_string(); }
        // bzip2: BZh
        if data.starts_with(b"BZh") { return "archives".to_string(); }
        // Windows PE (exe/dll): MZ
        if data.starts_with(b"MZ") { return "dangerous".to_string(); }
        // ELF (Linux executable)
        if data.starts_with(&[0x7F, 0x45, 0x4C, 0x46]) { return "dangerous".to_string(); }
    }
    // Fall back to extension+mime based classification
    classify_attachment_category(filename, mime, None, None)
}


#[tauri::command]
pub async fn email_attachments(state: State<'_, AppState>, input: Value) -> Result<Vec<Attachment>, String> {
    let email_id = input["email_id"].as_str()
        .or_else(|| input["emailId"].as_str())
        .or_else(|| input["input"]["email_id"].as_str())
        .or_else(|| input["input"]["emailId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare(
        "SELECT id, email_id, filename, sha256, mime_type, size_bytes, stored_path, entropy, risk_flags 
         FROM attachments WHERE email_id = ?1"
    ).map_err(|e| e.to_string())?;

    let attachments = stmt.query_map([&email_id], |row| {
        let risk_flags_str: String = row.get::<_, Option<String>>(8)?.unwrap_or_else(|| "[]".to_string());

        Ok(Attachment {
            id: row.get(0)?,
            email_id: row.get(1)?,
            filename: row.get(2)?,
            sha256: row.get(3)?,
            mime_type: row.get(4)?,
            size_bytes: row.get::<_, i64>(5)? as u64,
            stored_path: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
            entropy: row.get(7)?,
            risk_flags: risk_flags_str,
        })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

    Ok(attachments)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttachmentCategoryCounts {
    pub all: usize,
    pub dangerous: usize,
    pub documents: usize,
    pub images: usize,
    pub archives: usize,
    pub media: usize,
}

#[tauri::command]
pub async fn case_attachments_summary(state: State<'_, AppState>, input: Value) -> Result<AttachmentCategoryCounts, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input["input"]["case_id"].as_str())
        .or_else(|| input["input"]["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let evidence_id = input["evidence_id"].as_str()
        .or_else(|| input["evidenceId"].as_str())
        .or_else(|| input["input"]["evidence_id"].as_str())
        .or_else(|| input["input"]["evidenceId"].as_str())
        .filter(|s| !s.trim().is_empty() && *s != "all");

    let db = state.db.lock().await;
    
    let mut counts = AttachmentCategoryCounts {
        all: 0,
        dangerous: 0,
        documents: 0,
        images: 0,
        archives: 0,
        media: 0,
    };

    if let Some(ev_id) = evidence_id {
        let mut stmt = db.conn.prepare(
            "SELECT a.filename, a.mime_type, a.entropy, a.risk_flags
             FROM attachments a
             JOIN emails e ON a.email_id = e.id
             WHERE e.case_id = ?1 AND e.evidence_id = ?2"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map([&case_id, ev_id], |row| {
            let filename: String = row.get::<_, Option<String>>(0)?.unwrap_or_else(|| "attachment.bin".to_string());
            let mime: String = row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "application/octet-stream".to_string());
            let entropy: Option<f64> = row.get(2)?;
            let risk_flags: Option<String> = row.get(3)?;
            Ok(classify_attachment_category(&filename, &mime, entropy, risk_flags.as_deref()))
        }).map_err(|e| e.to_string())?;

        for r in rows.flatten() {
            counts.all += 1;
            match r.as_str() {
                "dangerous" => counts.dangerous += 1,
                "images" => counts.images += 1,
                "documents" => counts.documents += 1,
                "archives" => counts.archives += 1,
                "media" => counts.media += 1,
                _ => counts.documents += 1,
            }
        }
    } else {
        let mut stmt = db.conn.prepare(
            "SELECT a.filename, a.mime_type, a.entropy, a.risk_flags
             FROM attachments a
             JOIN emails e ON a.email_id = e.id
             WHERE e.case_id = ?1"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map([&case_id], |row| {
            let filename: String = row.get::<_, Option<String>>(0)?.unwrap_or_else(|| "attachment.bin".to_string());
            let mime: String = row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "application/octet-stream".to_string());
            let entropy: Option<f64> = row.get(2)?;
            let risk_flags: Option<String> = row.get(3)?;
            Ok(classify_attachment_category(&filename, &mime, entropy, risk_flags.as_deref()))
        }).map_err(|e| e.to_string())?;

        for r in rows.flatten() {
            counts.all += 1;
            match r.as_str() {
                "dangerous" => counts.dangerous += 1,
                "images" => counts.images += 1,
                "documents" => counts.documents += 1,
                "archives" => counts.archives += 1,
                "media" => counts.media += 1,
                _ => counts.documents += 1,
            }
        }
    }

    Ok(counts)
}

#[tauri::command]
pub async fn case_attachments_list(state: State<'_, AppState>, input: Value) -> Result<Vec<CaseAttachmentItem>, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input["input"]["case_id"].as_str())
        .or_else(|| input["input"]["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let evidence_id = input["evidence_id"].as_str()
        .or_else(|| input["evidenceId"].as_str())
        .or_else(|| input["input"]["evidence_id"].as_str())
        .or_else(|| input["input"]["evidenceId"].as_str())
        .filter(|s| !s.trim().is_empty() && *s != "all");

    let category = input["category"].as_str()
        .or_else(|| input["input"]["category"].as_str())
        .unwrap_or("all");

    let search = input["search"].as_str()
        .or_else(|| input["input"]["search"].as_str())
        .unwrap_or("")
        .to_lowercase();

    let db = state.db.lock().await;

    let items = if let Some(ev_id) = evidence_id {
        let mut stmt = db.conn.prepare(
            "SELECT a.id, a.email_id, a.filename, a.sha256, a.mime_type, a.size_bytes, 
                    a.stored_path, a.entropy, a.risk_flags,
                    e.subject, e.from_addr, e.date_sent_utc, e.risk_score
             FROM attachments a
             JOIN emails e ON a.email_id = e.id
             WHERE e.case_id = ?1 AND e.evidence_id = ?2
             ORDER BY a.size_bytes DESC"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map([&case_id, ev_id], |row| {
            let filename: String = row.get::<_, Option<String>>(2)?.unwrap_or_else(|| "attachment.bin".to_string());
            let mime: String = row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "application/octet-stream".to_string());
            let entropy: Option<f64> = row.get(7)?;
            let risk_flags: Option<String> = row.get(8)?;

            let cat = classify_attachment_category(&filename, &mime, entropy, risk_flags.as_deref());

            Ok(CaseAttachmentItem {
                id: row.get(0)?,
                email_id: row.get(1)?,
                filename,
                sha256: row.get(3)?,
                mime_type: mime,
                size_bytes: row.get::<_, i64>(5)? as u64,
                stored_path: row.get(6)?,
                entropy,
                risk_flags,
                email_subject: row.get(9)?,
                email_from: row.get::<_, Option<String>>(10)?.unwrap_or_default(),
                email_date: row.get(11)?,
                email_risk_score: row.get::<_, Option<u8>>(12)?.unwrap_or(0),
                category: cat,
            })
        }).map_err(|e| e.to_string())?;
        rows.filter_map(|r| r.ok()).collect::<Vec<_>>()
    } else {
        let mut stmt = db.conn.prepare(
            "SELECT a.id, a.email_id, a.filename, a.sha256, a.mime_type, a.size_bytes, 
                    a.stored_path, a.entropy, a.risk_flags,
                    e.subject, e.from_addr, e.date_sent_utc, e.risk_score
             FROM attachments a
             JOIN emails e ON a.email_id = e.id
             WHERE e.case_id = ?1
             ORDER BY a.size_bytes DESC"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map([&case_id], |row| {
            let filename: String = row.get::<_, Option<String>>(2)?.unwrap_or_else(|| "attachment.bin".to_string());
            let mime: String = row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "application/octet-stream".to_string());
            let entropy: Option<f64> = row.get(7)?;
            let risk_flags: Option<String> = row.get(8)?;

            let cat = classify_attachment_category(&filename, &mime, entropy, risk_flags.as_deref());

            Ok(CaseAttachmentItem {
                id: row.get(0)?,
                email_id: row.get(1)?,
                filename,
                sha256: row.get(3)?,
                mime_type: mime,
                size_bytes: row.get::<_, i64>(5)? as u64,
                stored_path: row.get(6)?,
                entropy,
                risk_flags,
                email_subject: row.get(9)?,
                email_from: row.get::<_, Option<String>>(10)?.unwrap_or_default(),
                email_date: row.get(11)?,
                email_risk_score: row.get::<_, Option<u8>>(12)?.unwrap_or(0),
                category: cat,
            })
        }).map_err(|e| e.to_string())?;
        rows.filter_map(|r| r.ok()).collect::<Vec<_>>()
    };

    let filtered = items.into_iter().filter(|item| {
        if category != "all" && item.category != category {
            return false;
        }
        if !search.is_empty() {
            let f_m = item.filename.to_lowercase().contains(&search);
            let s_m = item.email_subject.as_deref().unwrap_or("").to_lowercase().contains(&search);
            let from_m = item.email_from.to_lowercase().contains(&search);
            if !f_m && !s_m && !from_m {
                return false;
            }
        }
        true
    }).collect();

    Ok(filtered)
}

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

    let (filename, stored_path): (String, Option<String>) = db.conn.query_row(
        "SELECT filename, stored_path FROM attachments WHERE id=?1",
        [attachment_id],
        |r| Ok((r.get::<_, Option<String>>(0)?.unwrap_or_else(|| "exported_file".to_string()), r.get(1)?)),
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
            return Ok(target_file.to_string_lossy().to_string());
        }
    }

    std::fs::write(&target_file, format!("Attachment Export Receipt: {}\nID: {}\nExtracted with J12 Forensic Suite.", filename, attachment_id))
        .map_err(|e| format!("Failed to write export: {}", e))?;

    Ok(target_file.to_string_lossy().to_string())
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

    let stored_path: Option<String> = if !direct_path.is_empty() {
        Some(direct_path.to_string())
    } else if !attachment_id.is_empty() {
        let db = state.db.lock().await;
        db.conn.query_row(
            "SELECT stored_path FROM attachments WHERE id=?1",
            [attachment_id],
            |r| r.get(0),
        ).ok()
    } else {
        None
    };

    if let Some(path_str) = stored_path {
        if !path_str.is_empty() {
            let path = std::path::Path::new(&path_str);
            if path.exists() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

                // ── PDF thumbnail via macOS Quick Look ──────────────────────────────
                if ext == "pdf" {
                    let tmp_dir = std::env::temp_dir().join(format!("j12_ql_{}", attachment_id));
                    let _ = std::fs::create_dir_all(&tmp_dir);

                    let output = std::process::Command::new("qlmanage")
                        .args(["-t", "-s", "300", "-o"])
                        .arg(&tmp_dir)
                        .arg(path)
                        .output();

                    if let Ok(out) = output {
                        if out.status.success() {
                            // qlmanage writes <filename>.png inside tmp_dir
                            if let Ok(entries) = std::fs::read_dir(&tmp_dir) {
                                for entry in entries.flatten() {
                                    let ep = entry.path();
                                    if ep.extension().map(|e| e == "png").unwrap_or(false) {
                                        if let Ok(data) = std::fs::read(&ep) {
                                            use base64::Engine;
                                            let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
                                            let _ = std::fs::remove_dir_all(&tmp_dir);
                                            return Ok(Some(format!("data:image/png;base64,{}", b64)));
                                        }
                                    }
                                }
                            }
                        }
                        let _ = std::fs::remove_dir_all(&tmp_dir);
                    }
                    return Ok(None);
                }

                // ── Image types: read and base64 encode directly ────────────────────
                let mime = match ext.as_str() {
                    "jpg" | "jpeg" => "image/jpeg",
                    "png"  => "image/png",
                    "gif"  => "image/gif",
                    "webp" => "image/webp",
                    "svg"  => "image/svg+xml",
                    "bmp"  => "image/bmp",
                    "ico"  => "image/x-icon",
                    "tif" | "tiff" => "image/tiff",
                    _ => return Ok(None), // non-image, non-pdf — no preview
                };

                if let Ok(data) = std::fs::read(path) {
                    use base64::Engine;
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
                    return Ok(Some(format!("data:{};base64,{}", mime, b64)));
                }
            }
        }
    }
    Ok(None)
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
    let (filename, stored_path): (String, Option<String>) = db.conn.query_row(
        "SELECT filename, stored_path FROM attachments WHERE id=?1",
        [attachment_id],
        |r| Ok((r.get::<_, Option<String>>(0)?.unwrap_or_else(|| "attachment".to_string()), r.get(1)?)),
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

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct InlineImageData {
    pub attachment_id: String,
    pub filename: String,
    pub mime_type: String,
    pub data_url: String,
}

/// Returns all image attachments for an email as base64 data URLs
/// so the frontend can resolve cid: references in the HTML body.
#[tauri::command]
pub async fn get_email_inline_images(
    state: tauri::State<'_, crate::AppState>,
    input: serde_json::Value,
) -> Result<Vec<InlineImageData>, String> {
    let email_id: String = match input["email_id"].as_str()
        .or_else(|| input["emailId"].as_str())
        .or_else(|| input["id"].as_str())
        .or_else(|| input["input"]["email_id"].as_str())
        .or_else(|| input["input"]["emailId"].as_str())
        .or_else(|| input["input"]["id"].as_str())
        .or_else(|| input.as_str())
    {
        Some(s) if !s.is_empty() => s.to_owned(),
        _ => return Ok(vec![]),
    };

    let rows: Vec<(String, String, String, Option<String>)> = {
        let db = state.db.lock().await;
        let mut stmt = db.conn.prepare(
            "SELECT id, filename, mime_type, stored_path FROM attachments
             WHERE email_id = ?1
               AND (mime_type LIKE 'image/%'
                    OR filename LIKE '%.jpg' OR filename LIKE '%.jpeg'
                    OR filename LIKE '%.png' OR filename LIKE '%.gif'
                    OR filename LIKE '%.webp' OR filename LIKE '%.bmp'
                    OR filename LIKE '%.tif' OR filename LIKE '%.tiff')"
        ).map_err(|e| e.to_string())?;

        let rows: Vec<_> = stmt.query_map([&email_id as &str], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
        rows
    };

    let mut results = Vec::new();
    for (att_id, filename, mime_type, stored_path) in rows {
        if let Some(path_str) = stored_path {
            if !path_str.is_empty() {
                let path = std::path::Path::new(&path_str);
                if path.exists() {
                    if let Ok(data) = std::fs::read(path) {
                        use base64::Engine;
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
                        let ext = path.extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                        let resolved_mime = if mime_type.starts_with("image/") {
                            mime_type.clone()
                        } else {
                            match ext.as_str() {
                                "jpg" | "jpeg" => "image/jpeg".to_string(),
                                "png"  => "image/png".to_string(),
                                "gif"  => "image/gif".to_string(),
                                "webp" => "image/webp".to_string(),
                                "bmp"  => "image/bmp".to_string(),
                                _ => "image/png".to_string(),
                            }
                        };
                        results.push(InlineImageData {
                            attachment_id: att_id,
                            filename,
                            mime_type: resolved_mime.clone(),
                            data_url: format!("data:{};base64,{}", resolved_mime, b64),
                        });
                    }
                }
            }
        }
    }
    Ok(results)
}
