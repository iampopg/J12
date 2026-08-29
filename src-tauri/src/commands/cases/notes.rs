use chrono::Utc;
use tauri::State;

use crate::AppState;
use crate::db::generate_id;
use crate::models::*;
use super::super::helpers::boolv;

#[tauri::command]
pub async fn case_notes_list(state: State<'_, AppState>, input: EmptyInput) -> Result<Vec<CaseNote>, String> {
    let db = state.db.lock().await;
    let mut stmt = db.conn.prepare(
        "SELECT id, case_id, author, title, content, is_pinned, created_at, updated_at
         FROM case_notes
         WHERE case_id = ?1
         ORDER BY is_pinned DESC, created_at DESC"
    ).map_err(|e| e.to_string())?;

    let notes = stmt.query_map([&input.case_id], |row| {
        Ok(CaseNote {
            id: row.get(0)?,
            case_id: row.get(1)?,
            author: row.get(2)?,
            title: row.get(3)?,
            content: row.get(4)?,
            category: "general".to_string(),
            pinned: boolv(row, 5),
            created_at: row.get::<_, String>(6)?,
            updated_at: row.get::<_, String>(7)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(notes)
}

#[tauri::command]
pub async fn case_note_create(state: State<'_, AppState>, input: CaseNoteCreateInput) -> Result<CaseNote, String> {
    let db = state.db.lock().await;
    let id = generate_id();
    let now = Utc::now().to_rfc3339();
    let author = input.author.unwrap_or_else(|| "Examiner".to_string());
    let is_pinned = input.pinned.unwrap_or(false);
    let category = input.category.unwrap_or_else(|| "general".to_string());

    db.conn.execute(
        "INSERT INTO case_notes (id, case_id, author, title, content, is_pinned, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        rusqlite::params![
            id,
            input.case_id,
            author,
            input.title,
            input.content,
            is_pinned,
            now
        ],
    ).map_err(|e| e.to_string())?;

    Ok(CaseNote {
        id,
        case_id: input.case_id,
        author,
        title: input.title,
        content: input.content,
        category,
        pinned: is_pinned,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn case_note_update(state: State<'_, AppState>, input: CaseNoteUpdateInput) -> Result<(), String> {
    let db = state.db.lock().await;
    let now = Utc::now().to_rfc3339();

    db.conn.execute(
        "UPDATE case_notes SET
            title = COALESCE(?1, title),
            content = COALESCE(?2, content),
            is_pinned = COALESCE(?3, is_pinned),
            updated_at = ?4
         WHERE id = ?5",
        rusqlite::params![
            input.title,
            input.content,
            input.pinned,
            now,
            input.id
        ],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn case_note_toggle_pin(state: State<'_, AppState>, note_id: String) -> Result<bool, String> {
    let db = state.db.lock().await;
    let current_pinned: bool = db.conn.query_row(
        "SELECT is_pinned FROM case_notes WHERE id = ?1",
        [&note_id],
        |row| Ok(boolv(row, 0)),
    ).map_err(|e| e.to_string())?;

    let new_pinned = !current_pinned;
    let now = Utc::now().to_rfc3339();

    db.conn.execute(
        "UPDATE case_notes SET is_pinned = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![new_pinned, now, note_id],
    ).map_err(|e| e.to_string())?;

    Ok(new_pinned)
}

#[tauri::command]
pub async fn case_note_delete(state: State<'_, AppState>, note_id: String) -> Result<(), String> {
    let db = state.db.lock().await;
    db.conn.execute("DELETE FROM case_notes WHERE id = ?1", [&note_id]).map_err(|e| e.to_string())?;
    Ok(())
}
