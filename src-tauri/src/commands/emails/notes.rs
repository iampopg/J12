use chrono::Utc;
use tauri::State;

use crate::AppState;
use crate::db::generate_id;
use crate::models::{EmailNote, EmailNoteInput};

#[tauri::command]
pub async fn email_notes_list(
    state: State<'_, AppState>,
    email_id: Option<String>,
    #[allow(non_snake_case)] emailId: Option<String>,
) -> Result<Vec<EmailNote>, String> {
    let target_id = email_id.or(emailId).unwrap_or_default();
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare(
        "SELECT id, case_id, email_id, author, COALESCE(note, content, ''), created_at, updated_at
         FROM email_notes
         WHERE email_id = ?1
         ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;

    let notes = stmt.query_map([&target_id], |row| {
        Ok(EmailNote {
            id: row.get(0)?,
            case_id: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            email_id: row.get(2)?,
            author: row.get(3)?,
            content: row.get(4)?,
            created_at: row.get::<_, String>(5)?,
            updated_at: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(notes)
}

#[tauri::command]
pub async fn email_note_add(state: State<'_, AppState>, input: EmailNoteInput) -> Result<EmailNote, String> {
    let db = state.db.lock().await;
    let id = generate_id();
    let now = Utc::now().to_rfc3339();
    let author = input.author.unwrap_or_else(|| "Examiner".to_string());

    db.conn.execute(
        "INSERT INTO email_notes (id, case_id, email_id, author, note, content, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?6)",
        rusqlite::params![id, input.case_id, input.email_id, author, input.content, now],
    ).map_err(|e| e.to_string())?;

    crate::audit_logger::log_forensic_event(
        &input.case_id,
        "CASE_NOTES",
        "NOTE_CREATED",
        &author,
        None,
        None,
        &format!("Added forensic note to email [{}]", input.email_id)
    );

    Ok(EmailNote {
        id,
        case_id: input.case_id,
        email_id: input.email_id,
        author,
        content: input.content,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn email_note_delete(
    state: State<'_, AppState>,
    note_id: Option<String>,
    #[allow(non_snake_case)] noteId: Option<String>,
) -> Result<(), String> {
    let target_id = note_id.or(noteId).unwrap_or_default();
    let db = state.db.lock().await;
    db.conn.execute("DELETE FROM email_notes WHERE id = ?1", [&target_id]).map_err(|e| e.to_string())?;
    Ok(())
}
