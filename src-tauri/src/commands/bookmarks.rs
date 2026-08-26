use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;
use crate::AppState;
use crate::db::generate_id;
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemBookmark {
    pub id: String,
    pub case_id: String,
    pub item_id: String,
    pub item_type: String,  // "email" | "attachment" | "finding"
    pub label: String,
    pub color: String,
    pub note: String,
    pub created_at: String,
    // Joined fields for display
    pub item_title: Option<String>,
    pub item_from: Option<String>,
    pub item_date: Option<String>,
}

#[tauri::command]
pub async fn bookmark_add(
    state: State<'_, AppState>,
    input: Value,
) -> Result<ItemBookmark, String> {
    let case_id = input["case_id"].as_str().unwrap_or("").to_string();
    let item_id = input["item_id"].as_str().unwrap_or("").to_string();
    let item_type = input["item_type"].as_str().unwrap_or("email").to_string();
    let label = input["label"].as_str().unwrap_or("Bookmarked").to_string();
    let color = input["color"].as_str().unwrap_or("#3b82f6").to_string();
    let note = input["note"].as_str().unwrap_or("").to_string();

    if case_id.is_empty() || item_id.is_empty() {
        return Err("case_id and item_id are required".to_string());
    }

    let id = generate_id();
    let now = Utc::now().to_rfc3339();

    let db = state.db.lock().await;
    db.conn.execute(
        "INSERT INTO item_bookmarks (id, case_id, item_id, item_type, label, color, note, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(case_id, item_id) DO UPDATE SET label=excluded.label, color=excluded.color, note=excluded.note",
        rusqlite::params![id, case_id, item_id, item_type, label, color, note, now],
    ).map_err(|e| e.to_string())?;

    Ok(ItemBookmark {
        id,
        case_id,
        item_id,
        item_type,
        label,
        color,
        note,
        created_at: now,
        item_title: None,
        item_from: None,
        item_date: None,
    })
}

#[tauri::command]
pub async fn bookmark_remove(
    state: State<'_, AppState>,
    input: Value,
) -> Result<(), String> {
    let item_id = input["item_id"].as_str().unwrap_or("").to_string();
    let case_id = input["case_id"].as_str().unwrap_or("").to_string();
    if item_id.is_empty() { return Err("item_id required".to_string()); }
    let db = state.db.lock().await;
    if case_id.is_empty() {
        db.conn.execute("DELETE FROM item_bookmarks WHERE item_id = ?1", [&item_id])
            .map_err(|e| e.to_string())?;
    } else {
        db.conn.execute("DELETE FROM item_bookmarks WHERE item_id = ?1 AND case_id = ?2",
            rusqlite::params![item_id, case_id])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn bookmarks_list(
    state: State<'_, AppState>,
    input: Value,
) -> Result<Vec<ItemBookmark>, String> {
    let case_id: String = match input["case_id"].as_str()
        .or_else(|| input.as_str())
    {
        Some(s) if !s.is_empty() => s.to_owned(),
        _ => return Ok(vec![]),
    };

    let db = state.db.lock().await;

    // Fetch all bookmarks for this case, joining item metadata where possible
    let mut stmt = db.conn.prepare(
        "SELECT
            b.id, b.case_id, b.item_id, b.item_type, b.label, b.color, b.note, b.created_at,
            CASE b.item_type
                WHEN 'email'      THEN e.subject
                WHEN 'attachment' THEN a.filename
                WHEN 'finding'    THEN f.title
                WHEN 'artifact'   THEN COALESCE(f.title, b.item_id)
                ELSE NULL
            END as item_title,
            CASE b.item_type
                WHEN 'email'      THEN e.from_addr
                WHEN 'attachment' THEN e2.from_addr
                WHEN 'finding'    THEN NULL
                WHEN 'artifact'   THEN NULL
                ELSE NULL
            END as item_from,
            CASE b.item_type
                WHEN 'email'      THEN e.date_sent_utc
                WHEN 'attachment' THEN e2.date_sent_utc
                ELSE NULL
            END as item_date
         FROM item_bookmarks b
         LEFT JOIN emails e      ON b.item_type = 'email'      AND b.item_id = e.id
         LEFT JOIN attachments a ON b.item_type = 'attachment' AND b.item_id = a.id
         LEFT JOIN emails e2     ON b.item_type = 'attachment' AND a.email_id = e2.id
         LEFT JOIN findings f    ON (b.item_type = 'finding' OR b.item_type = 'artifact') AND b.item_id = f.id
         WHERE b.case_id = ?1
         ORDER BY b.created_at DESC"
    ).map_err(|e| e.to_string())?;

    let rows: Vec<_> = stmt.query_map([&case_id as &str], |row| {
        Ok(ItemBookmark {
            id: row.get(0)?,
            case_id: row.get(1)?,
            item_id: row.get(2)?,
            item_type: row.get(3)?,
            label: row.get(4)?,
            color: row.get(5)?,
            note: row.get(6)?,
            created_at: row.get(7)?,
            item_title: row.get(8)?,
            item_from: row.get(9)?,
            item_date: row.get(10)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    Ok(rows)
}

#[tauri::command]
pub async fn bookmark_check(
    state: State<'_, AppState>,
    input: Value,
) -> Result<Option<ItemBookmark>, String> {
    let case_id: String = input["case_id"].as_str().unwrap_or("").to_owned();
    let item_id: String = input["item_id"].as_str().unwrap_or("").to_owned();
    if case_id.is_empty() || item_id.is_empty() { return Ok(None); }

    let db = state.db.lock().await;
    let r = db.conn.query_row(
        "SELECT id, case_id, item_id, item_type, label, color, note, created_at
         FROM item_bookmarks WHERE case_id = ?1 AND item_id = ?2",
        rusqlite::params![case_id, item_id],
        |row| Ok(ItemBookmark {
            id: row.get(0)?,
            case_id: row.get(1)?,
            item_id: row.get(2)?,
            item_type: row.get(3)?,
            label: row.get(4)?,
            color: row.get(5)?,
            note: row.get(6)?,
            created_at: row.get(7)?,
            item_title: None, item_from: None, item_date: None,
        }),
    );
    match r {
        Ok(b) => Ok(Some(b)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}
