use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

use crate::AppState;
use crate::analysis::doc_extractor::extract_document_text;
use crate::analysis::ocr_engine::extract_image_ocr;

#[derive(Debug, Serialize, Deserialize)]
pub struct AttachmentExtractionResult {
    pub attachment_id: String,
    pub extracted_text: String,
    pub ocr_status: String,
    pub char_count: usize,
}

#[tauri::command]
pub async fn extract_attachment_text(
    state: State<'_, AppState>,
    attachment_id: String,
) -> Result<AttachmentExtractionResult, String> {
    let db = state.db.lock().await;

    let (stored_path, filename, mime_type, email_id, case_id): (Option<String>, String, Option<String>, String, String) = db.conn.query_row(
        "SELECT a.stored_path, a.filename, a.mime_type, a.email_id, e.case_id 
         FROM attachments a 
         JOIN emails e ON a.email_id = e.id 
         WHERE a.id = ?1",
        rusqlite::params![&attachment_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
    ).map_err(|e| format!("Attachment not found: {}", e))?;

    let path_str = stored_path.ok_or_else(|| "Attachment file has no stored path on disk".to_string())?;
    let path = Path::new(&path_str);
    if !path.exists() {
        return Err(format!("Attachment file not found at path: {}", path_str));
    }

    // Step 1: Try document text extraction (PDF, DOCX, XLSX, PPTX, RTF, CSV, TXT)
    let mut text = extract_document_text(path, &filename, mime_type.as_deref()).unwrap_or_default();
    let mut ocr_status = "extracted".to_string();

    // Step 2: If text is empty, try OCR (for images or scanned PDFs)
    if text.trim().is_empty() {
        if let Ok(ocr_text) = extract_image_ocr(path, &filename) {
            if !ocr_text.trim().is_empty() {
                text = ocr_text;
                ocr_status = "ocr_completed".to_string();
            }
        }
    }

    if text.trim().is_empty() {
        ocr_status = "no_text_found".to_string();
    }

    // Step 3: Save to SQLite attachments table
    let _ = db.conn.execute(
        "UPDATE attachments SET extracted_text = ?1, ocr_status = ?2 WHERE id = ?3",
        rusqlite::params![&text, &ocr_status, &attachment_id],
    );

    // Step 4: Update FTS5 index for the email with this attachment text
    let _ = db.conn.execute(
        "UPDATE emails_fts SET attachment_text = attachment_text || ' ' || ?1 WHERE email_id = ?2",
        rusqlite::params![&text, &email_id],
    );

    crate::audit_logger::log_forensic_event(
        &case_id,
        "ATTACHMENT_TEXT_EXTRACTED",
        "DOC_OCR_PIPELINE",
        "Examiner",
        None,
        Some(&attachment_id),
        &format!("Extracted {} characters from attachment '{}' (status: {})", text.len(), filename, ocr_status),
    );

    Ok(AttachmentExtractionResult {
        attachment_id,
        extracted_text: text.clone(),
        ocr_status,
        char_count: text.len(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchExtractionSummary {
    pub total_processed: usize,
    pub successful: usize,
    pub failed: usize,
}

#[tauri::command]
pub async fn batch_extract_case_attachments(
    state: State<'_, AppState>,
    case_id: String,
) -> Result<BatchExtractionSummary, String> {
    let db = state.db.lock().await;

    let mut stmt = db.conn.prepare(
        "SELECT a.id, a.stored_path, a.filename, a.mime_type, a.email_id 
         FROM attachments a 
         JOIN emails e ON a.email_id = e.id 
         WHERE e.case_id = ?1 AND (a.extracted_text IS NULL OR a.extracted_text = '')"
    ).map_err(|e| e.to_string())?;

    let rows: Vec<(String, Option<String>, String, Option<String>, String)> = stmt.query_map([&case_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    let mut successful = 0;
    let mut failed = 0;

    for (att_id, stored_path, filename, mime_type, email_id) in rows {
        if let Some(p) = stored_path {
            let path = Path::new(&p);
            if path.exists() {
                let mut text = extract_document_text(path, &filename, mime_type.as_deref()).unwrap_or_default();
                let mut ocr_status = "extracted".to_string();

                if text.trim().is_empty() {
                    if let Ok(ocr_text) = extract_image_ocr(path, &filename) {
                        if !ocr_text.trim().is_empty() {
                            text = ocr_text;
                            ocr_status = "ocr_completed".to_string();
                        }
                    }
                }

                if text.trim().is_empty() {
                    ocr_status = "no_text_found".to_string();
                }

                let _ = db.conn.execute(
                    "UPDATE attachments SET extracted_text = ?1, ocr_status = ?2 WHERE id = ?3",
                    rusqlite::params![&text, &ocr_status, &att_id],
                );

                if !text.is_empty() {
                    let _ = db.conn.execute(
                        "UPDATE emails_fts SET attachment_text = attachment_text || ' ' || ?1 WHERE email_id = ?2",
                        rusqlite::params![&text, &email_id],
                    );
                }

                successful += 1;
                continue;
            }
        }
        failed += 1;
    }

    Ok(BatchExtractionSummary {
        total_processed: successful + failed,
        successful,
        failed,
    })
}
