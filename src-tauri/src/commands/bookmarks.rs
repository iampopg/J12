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
    let case_id: String = input["case_id"].as_str()
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
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s != "all");

    if case_id.is_empty() {
        return Ok(vec![]);
    }

    let db = state.db.lock().await;

    // Fetch all bookmarks for this case/evidence, joining item metadata where possible
    let mut rows: Vec<ItemBookmark> = if let Some(ref ev_id) = evidence_id {
        let mut stmt = db.conn.prepare(
            "SELECT
                b.id, b.case_id, b.item_id, b.item_type, b.label, b.color, b.note, b.created_at,
                CASE b.item_type
                    WHEN 'email'      THEN e.subject
                    WHEN 'attachment' THEN a.filename
                    WHEN 'finding'    THEN f.title
                    WHEN 'artifact'   THEN COALESCE(fa.title || ': ' || fa.primary_value, f.title, b.item_id)
                    ELSE NULL
                END as item_title,
                CASE b.item_type
                    WHEN 'email'      THEN e.from_addr
                    WHEN 'attachment' THEN e2.from_addr
                    WHEN 'finding'    THEN NULL
                    WHEN 'artifact'   THEN COALESCE(fa.email_from, e3.from_addr)
                    ELSE NULL
                END as item_from,
                CASE b.item_type
                    WHEN 'email'      THEN e.date_sent_utc
                    WHEN 'attachment' THEN e2.date_sent_utc
                    WHEN 'finding'    THEN NULL
                    WHEN 'artifact'   THEN COALESCE(fa.date_sent_utc, e3.date_sent_utc)
                    ELSE NULL
                END as item_date
             FROM item_bookmarks b
             LEFT JOIN emails e              ON b.item_type = 'email'      AND b.item_id = e.id
             LEFT JOIN attachments a         ON b.item_type = 'attachment' AND b.item_id = a.id
             LEFT JOIN emails e2             ON b.item_type = 'attachment' AND a.email_id = e2.id
             LEFT JOIN findings f            ON b.item_type = 'finding'    AND b.item_id = f.id
             LEFT JOIN forensic_artifacts fa ON b.item_type = 'artifact'   AND b.item_id = fa.id
             LEFT JOIN emails e3             ON b.item_type = 'artifact'   AND fa.email_id = e3.id
             WHERE b.case_id = ?1
               AND (
                 (b.item_type = 'email' AND e.evidence_id = ?2)
                 OR (b.item_type = 'attachment' AND (e2.evidence_id = ?2 OR a.email_id IN (SELECT id FROM emails WHERE evidence_id = ?2)))
                 OR (b.item_type = 'finding' AND (f.evidence_refs LIKE '%' || ?2 || '%' OR EXISTS (SELECT 1 FROM emails em WHERE em.evidence_id = ?2 AND instr(f.email_ids, em.id) > 0)))
                 OR (b.item_type = 'artifact' AND (fa.email_id IN (SELECT id FROM emails WHERE evidence_id = ?2) OR e3.evidence_id = ?2))
               )
             ORDER BY b.created_at DESC"
        ).map_err(|e| e.to_string())?;

        let res: Vec<ItemBookmark> = stmt.query_map(rusqlite::params![&case_id, ev_id], |row| {
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
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    } else {
        let mut stmt = db.conn.prepare(
            "SELECT
                b.id, b.case_id, b.item_id, b.item_type, b.label, b.color, b.note, b.created_at,
                CASE b.item_type
                    WHEN 'email'      THEN e.subject
                    WHEN 'attachment' THEN a.filename
                    WHEN 'finding'    THEN f.title
                    WHEN 'artifact'   THEN COALESCE(fa.title || ': ' || fa.primary_value, f.title, b.item_id)
                    ELSE NULL
                END as item_title,
                CASE b.item_type
                    WHEN 'email'      THEN e.from_addr
                    WHEN 'attachment' THEN e2.from_addr
                    WHEN 'finding'    THEN NULL
                    WHEN 'artifact'   THEN COALESCE(fa.email_from, e3.from_addr)
                    ELSE NULL
                END as item_from,
                CASE b.item_type
                    WHEN 'email'      THEN e.date_sent_utc
                    WHEN 'attachment' THEN e2.date_sent_utc
                    WHEN 'finding'    THEN NULL
                    WHEN 'artifact'   THEN COALESCE(fa.date_sent_utc, e3.date_sent_utc)
                    ELSE NULL
                END as item_date
             FROM item_bookmarks b
             LEFT JOIN emails e              ON b.item_type = 'email'      AND b.item_id = e.id
             LEFT JOIN attachments a         ON b.item_type = 'attachment' AND b.item_id = a.id
             LEFT JOIN emails e2             ON b.item_type = 'attachment' AND a.email_id = e2.id
             LEFT JOIN findings f            ON b.item_type = 'finding'    AND b.item_id = f.id
             LEFT JOIN forensic_artifacts fa ON b.item_type = 'artifact'   AND b.item_id = fa.id
             LEFT JOIN emails e3             ON b.item_type = 'artifact'   AND fa.email_id = e3.id
             WHERE b.case_id = ?1
             ORDER BY b.created_at DESC"
        ).map_err(|e| e.to_string())?;

        let res: Vec<ItemBookmark> = stmt.query_map([&case_id as &str], |row| {
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
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    };

    // Also fetch email_tags to ensure all tagged emails appear in the Tagged Evidence Locker
    let email_tag_rows: Vec<ItemBookmark> = if let Some(ref ev_id) = evidence_id {
        let mut email_tag_stmt = db.conn.prepare(
            "SELECT t.id, t.case_id, t.email_id, t.tag, t.color, t.created_at, e.subject, e.from_addr, e.date_sent_utc
             FROM email_tags t
             JOIN emails e ON t.email_id = e.id
             WHERE (t.case_id = ?1 OR e.case_id = ?1) AND e.evidence_id = ?2
             ORDER BY t.created_at DESC"
        ).map_err(|e| e.to_string())?;

        let res: Vec<ItemBookmark> = email_tag_stmt.query_map(rusqlite::params![&case_id, ev_id], |row| {
            Ok(ItemBookmark {
                id: row.get(0)?,
                case_id: row.get(1)?,
                item_id: row.get(2)?,
                item_type: "email".to_string(),
                label: row.get(3)?,
                color: row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "#3b82f6".to_string()),
                note: "Tagged Email".to_string(),
                created_at: row.get(5)?,
                item_title: row.get(6)?,
                item_from: row.get(7)?,
                item_date: row.get(8)?,
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    } else {
        let mut email_tag_stmt = db.conn.prepare(
            "SELECT t.id, t.case_id, t.email_id, t.tag, t.color, t.created_at, e.subject, e.from_addr, e.date_sent_utc
             FROM email_tags t
             JOIN emails e ON t.email_id = e.id
             WHERE t.case_id = ?1 OR e.case_id = ?1
             ORDER BY t.created_at DESC"
        ).map_err(|e| e.to_string())?;

        let res: Vec<ItemBookmark> = email_tag_stmt.query_map([&case_id as &str], |row| {
            Ok(ItemBookmark {
                id: row.get(0)?,
                case_id: row.get(1)?,
                item_id: row.get(2)?,
                item_type: "email".to_string(),
                label: row.get(3)?,
                color: row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "#3b82f6".to_string()),
                note: "Tagged Email".to_string(),
                created_at: row.get(5)?,
                item_title: row.get(6)?,
                item_from: row.get(7)?,
                item_date: row.get(8)?,
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    };

    let mut existing_set = std::collections::HashSet::new();
    for r in &rows {
        existing_set.insert(format!("{}:{}", r.item_id, r.label.to_lowercase()));
    }

    for et in email_tag_rows {
        let key = format!("{}:{}", et.item_id, et.label.to_lowercase());
        if !existing_set.contains(&key) {
            existing_set.insert(key);
            rows.push(et);
        }
    }

    // Sort all rows newest first
    rows.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(rows)
}

#[tauri::command]
pub async fn bookmark_check(
    state: State<'_, AppState>,
    input: Value,
) -> Result<Option<ItemBookmark>, String> {
    let case_id: String = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input["input"]["case_id"].as_str())
        .or_else(|| input["input"]["caseId"].as_str())
        .unwrap_or("")
        .to_string();

    let item_id: String = input["item_id"].as_str()
        .or_else(|| input["itemId"].as_str())
        .or_else(|| input["input"]["item_id"].as_str())
        .or_else(|| input["input"]["itemId"].as_str())
        .unwrap_or("")
        .to_string();

    if case_id.is_empty() || item_id.is_empty() {
        return Ok(None);
    }

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
