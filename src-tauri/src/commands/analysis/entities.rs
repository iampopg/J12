use std::collections::HashMap;
use chrono::Utc;
use serde_json::Value;
use tauri::State;

use crate::AppState;
use crate::db::generate_id;
use crate::models::*;
use super::super::helpers::{boolv, u8v};

#[tauri::command]
pub async fn extract_entities(state: State<'_, AppState>, input: Value) -> Result<u32, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let db = state.db.lock().await;

    let mut stmt = db.conn.prepare(
        "SELECT from_addr, to_addrs, cc_addrs, bcc_addrs 
         FROM emails WHERE case_id = ?1"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([&case_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

    let mut entity_map: HashMap<String, (i64, i64)> = HashMap::new();
    let re_email = regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();

    for (from, to, cc, bcc) in rows {
        for cap in re_email.captures_iter(&from) {
            let val = cap[0].to_lowercase();
            let entry = entity_map.entry(val).or_insert((0, 0));
            entry.0 += 1;
        }

        let parse_field = |field_str: &str| -> Vec<String> {
            if let Ok(arr) = serde_json::from_str::<Vec<String>>(field_str) {
                arr
            } else {
                re_email.captures_iter(field_str).map(|c| c[0].to_string()).collect()
            }
        };

        for r_raw in parse_field(&to).into_iter().chain(parse_field(&cc)).chain(parse_field(&bcc)) {
            for cap in re_email.captures_iter(&r_raw) {
                let val = cap[0].to_lowercase();
                let entry = entity_map.entry(val).or_insert((0, 0));
                entry.1 += 1;
            }
        }
    }

    let mut count = 0;
    for (email_addr, (sent, recv)) in entity_map {
        let id = generate_id();
        let first_seen = Utc::now().to_rfc3339();

        let _ = db.conn.execute(
            "INSERT OR REPLACE INTO entities (id, case_id, email_address, display_name, first_seen, last_seen, sent_count, received_count, role, aliases)
             VALUES (?1, ?2, ?3, ?3, ?4, ?4, ?5, ?6, 'participant', '[]')",
            rusqlite::params![id, case_id, email_addr, first_seen, sent, recv]
        );
        count += 1;
    }

    crate::audit_logger::log_forensic_event(
        &case_id,
        "ENTITY_EXTRACTION",
        "ENTITIES_INDEXED",
        "System Pipeline",
        None,
        None,
        &format!("Extracted & indexed {} communication entities/participants", count)
    );

    Ok(count)
}

#[tauri::command]
pub async fn entity_list(state: State<'_, AppState>, input: EmptyInput) -> Result<Vec<Entity>, String> {
    let db = state.db.lock().await;
    let cid = &input.case_id;

    if let Some(ref ev_id) = input.evidence_id.as_ref().filter(|s| !s.is_empty() && *s != "all") {
        let mut stmt = db.conn.prepare(
            "SELECT from_addr, to_addrs, cc_addrs, bcc_addrs 
             FROM emails WHERE case_id = ?1 AND evidence_id = ?2"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map(rusqlite::params![cid, ev_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        let mut entity_map: HashMap<String, (i64, i64)> = HashMap::new();
        let re_email = regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();

        for (from, to, cc, bcc) in rows {
            for cap in re_email.captures_iter(&from) {
                let val = cap[0].to_lowercase();
                let entry = entity_map.entry(val).or_insert((0, 0));
                entry.0 += 1;
            }
            let recipients = format!("{} {} {}", to, cc, bcc);
            for cap in re_email.captures_iter(&recipients) {
                let val = cap[0].to_lowercase();
                let entry = entity_map.entry(val).or_insert((0, 0));
                entry.1 += 1;
            }
        }

        let mut entities: Vec<Entity> = entity_map.into_iter().map(|(email_addr, (sent, recv))| {
            let display_name = email_addr.split('@').next().unwrap_or(&email_addr).to_string();
            Entity {
                id: format!("ent_{}", email_addr),
                case_id: cid.clone(),
                email_address: email_addr,
                display_name: Some(display_name),
                first_seen: None,
                last_seen: None,
                sent_count: sent,
                received_count: recv,
                role: "participant".to_string(),
                aliases: Some("[]".to_string()),
            }
        }).collect();

        entities.sort_by(|a, b| (b.sent_count + b.received_count).cmp(&(a.sent_count + a.received_count)));
        Ok(entities)
    } else {
        let mut stmt = db.conn.prepare(
            "SELECT id, case_id, email_address, display_name, first_seen, last_seen, sent_count, received_count, role, aliases
             FROM entities WHERE case_id = ?1 ORDER BY (sent_count + received_count) DESC"
        ).map_err(|e| e.to_string())?;

        let entities = stmt.query_map([cid], |row| {
            Ok(Entity {
                id: row.get(0)?,
                case_id: row.get(1)?,
                email_address: row.get(2)?,
                display_name: row.get(3)?,
                first_seen: row.get(4)?,
                last_seen: row.get(5)?,
                sent_count: row.get(6)?,
                received_count: row.get(7)?,
                role: row.get(8)?,
                aliases: row.get(9)?,
            })
        }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

        Ok(entities)
    }
}

#[tauri::command]
pub async fn entity_dive(state: State<'_, AppState>, input: EntityInput) -> Result<Value, String> {
    let db = state.db.lock().await;

    let email_like = format!("%{}%", input.email_address);

    let (sent_count, recv_count, deleted_count, flagged_count, first_seen, last_seen) = if let Some(ref ev_id) = input.evidence_id.as_ref().filter(|s| !s.is_empty() && *s != "all") {
        let sent: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND from_addr LIKE ?3",
            rusqlite::params![&input.case_id, ev_id, &email_like],
            |r| r.get(0)
        ).unwrap_or(0);

        let recv: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND (to_addrs LIKE ?3 OR cc_addrs LIKE ?3)",
            rusqlite::params![&input.case_id, ev_id, &email_like],
            |r| r.get(0)
        ).unwrap_or(0);

        let del: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND (from_addr LIKE ?3 OR to_addrs LIKE ?3 OR cc_addrs LIKE ?3) AND is_deleted=1",
            rusqlite::params![&input.case_id, ev_id, &email_like],
            |r| r.get(0)
        ).unwrap_or(0);

        let flag: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND (from_addr LIKE ?3 OR to_addrs LIKE ?3 OR cc_addrs LIKE ?3) AND risk_score >= 50",
            rusqlite::params![&input.case_id, ev_id, &email_like],
            |r| r.get(0)
        ).unwrap_or(0);

        let (first, last): (Option<String>, Option<String>) = db.conn.query_row(
            "SELECT MIN(date_sent_utc), MAX(date_sent_utc) FROM emails WHERE case_id=?1 AND evidence_id=?2 AND (from_addr LIKE ?3 OR to_addrs LIKE ?3 OR cc_addrs LIKE ?3)",
            rusqlite::params![&input.case_id, ev_id, &email_like],
            |r| Ok((r.get(0)?, r.get(1)?))
        ).unwrap_or((None, None));

        (sent, recv, del, flag, first, last)
    } else {
        let sent: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND from_addr LIKE ?2",
            rusqlite::params![&input.case_id, &email_like],
            |r| r.get(0)
        ).unwrap_or(0);

        let recv: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND (to_addrs LIKE ?2 OR cc_addrs LIKE ?2)",
            rusqlite::params![&input.case_id, &email_like],
            |r| r.get(0)
        ).unwrap_or(0);

        let del: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR cc_addrs LIKE ?2) AND is_deleted=1",
            rusqlite::params![&input.case_id, &email_like],
            |r| r.get(0)
        ).unwrap_or(0);

        let flag: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR cc_addrs LIKE ?2) AND risk_score >= 50",
            rusqlite::params![&input.case_id, &email_like],
            |r| r.get(0)
        ).unwrap_or(0);

        let (first, last): (Option<String>, Option<String>) = db.conn.query_row(
            "SELECT MIN(date_sent_utc), MAX(date_sent_utc) FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR cc_addrs LIKE ?2)",
            rusqlite::params![&input.case_id, &email_like],
            |r| Ok((r.get(0)?, r.get(1)?))
        ).unwrap_or((None, None));

        (sent, recv, del, flag, first, last)
    };

    let display_name: Option<String> = db.conn.query_row(
        "SELECT display_name FROM entities WHERE case_id=?1 AND email_address=?2",
        rusqlite::params![&input.case_id, &input.email_address],
        |r| r.get(0)
    ).ok().flatten().or_else(|| {
        db.conn.query_row(
            "SELECT from_display FROM emails WHERE case_id=?1 AND from_addr LIKE ?2 AND from_display IS NOT NULL AND from_display != '' LIMIT 1",
            rusqlite::params![&input.case_id, &email_like],
            |r| r.get(0)
        ).ok()
    });

    let mut stmt_to = db.conn.prepare(
        "SELECT to_addrs, COUNT(*) as c FROM emails WHERE case_id=?1 AND from_addr LIKE ?2 AND to_addrs IS NOT NULL AND to_addrs != '' GROUP BY to_addrs ORDER BY c DESC LIMIT 15"
    ).map_err(|e| e.to_string())?;
    let sent_to: Vec<(String, i64)> = stmt_to.query_map(rusqlite::params![&input.case_id, &email_like], |r| {
        let raw: String = r.get(0)?;
        let clean = raw.trim_matches(|c| c == '[' || c == ']' || c == '"' || c == '\'').trim().to_string();
        Ok((clean, r.get(1)?))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    let mut stmt_from = db.conn.prepare(
        "SELECT from_addr, COUNT(*) as c FROM emails WHERE case_id=?1 AND (to_addrs LIKE ?2 OR cc_addrs LIKE ?2) AND from_addr IS NOT NULL AND from_addr != '' GROUP BY from_addr ORDER BY c DESC LIMIT 15"
    ).map_err(|e| e.to_string())?;
    let received_from: Vec<(String, i64)> = stmt_from.query_map(rusqlite::params![&input.case_id, &email_like], |r| {
        Ok((r.get(0)?, r.get(1)?))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    let mut stmt_subj = db.conn.prepare(
        "SELECT subject, COUNT(*) as c FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR cc_addrs LIKE ?2) AND subject IS NOT NULL AND subject != '' GROUP BY subject ORDER BY c DESC LIMIT 10"
    ).map_err(|e| e.to_string())?;
    let top_subjects: Vec<(String, i64)> = stmt_subj.query_map(rusqlite::params![&input.case_id, &email_like], |r| {
        Ok((r.get(0)?, r.get(1)?))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    let mut stmt_aliases = db.conn.prepare(
        "SELECT DISTINCT from_display FROM emails WHERE case_id=?1 AND from_addr LIKE ?2 AND from_display IS NOT NULL AND from_display != '' LIMIT 10"
    ).map_err(|e| e.to_string())?;
    let aliases: Vec<String> = stmt_aliases.query_map(rusqlite::params![&input.case_id, &email_like], |r| {
        r.get(0)
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    Ok(serde_json::json!({
        "email": input.email_address,
        "display_name": display_name,
        "first_seen": first_seen,
        "last_seen": last_seen,
        "sent_count": sent_count,
        "received_count": recv_count,
        "deleted_count": deleted_count,
        "flagged_count": flagged_count,
        "total_count": sent_count + recv_count,
        "aliases": aliases,
        "sent_to": sent_to,
        "received_from": received_from,
        "top_subjects": top_subjects,
    }))
}

#[tauri::command]
pub async fn entity_emails(state: State<'_, AppState>, input: Value) -> Result<Vec<EmailMessage>, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input["input"]["case_id"].as_str())
        .or_else(|| input["input"]["caseId"].as_str())
        .unwrap_or("");
    let email_addr = input["email_address"].as_str()
        .or_else(|| input["email"].as_str())
        .or_else(|| input["input"]["email_address"].as_str())
        .or_else(|| input["input"]["email"].as_str())
        .unwrap_or("");
    let evidence_id = input["evidence_id"].as_str()
        .or_else(|| input["evidenceId"].as_str())
        .or_else(|| input["input"]["evidence_id"].as_str())
        .or_else(|| input["input"]["evidenceId"].as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && *s != "all");

    let db = state.db.lock().await;
    let e_like = format!("%{}%", email_addr);

    let emails = if let Some(ev_id) = evidence_id {
        let mut stmt = db.conn.prepare(
            "SELECT id, evidence_id, case_id, message_id, from_addr, from_display, to_addrs, cc_addrs, subject,
                    date_sent, date_sent_utc, folder_name, folder_category,
                    is_deleted, deleted_recovered, risk_score, flags,
                    body_text, body_html, headers_raw
             FROM emails WHERE case_id=?1 AND evidence_id=?2 AND (from_addr LIKE ?3 OR to_addrs LIKE ?3 OR cc_addrs LIKE ?3)
             ORDER BY date_sent_utc DESC LIMIT 1000"
        ).map_err(|e| e.to_string())?;

        let res: Vec<EmailMessage> = stmt.query_map(rusqlite::params![case_id, ev_id, e_like], |row| {
            Ok(EmailMessage {
                id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?,
                from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?,
                subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?,
                headers_raw: row.get(19)?,
                body_text: row.get(17)?,
                body_html: row.get(18)?,
                folder_name: row.get(11)?, folder_category: row.get(12)?,
                is_deleted: boolv(row, 13), deleted_recovered: boolv(row, 14), risk_score: u8v(row, 15), flags: row.get(16)?
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    } else {
        let mut stmt = db.conn.prepare(
            "SELECT id, evidence_id, case_id, message_id, from_addr, from_display, to_addrs, cc_addrs, subject,
                    date_sent, date_sent_utc, folder_name, folder_category,
                    is_deleted, deleted_recovered, risk_score, flags,
                    body_text, body_html, headers_raw
             FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2 OR cc_addrs LIKE ?2)
             ORDER BY date_sent_utc DESC LIMIT 1000"
        ).map_err(|e| e.to_string())?;

        let res: Vec<EmailMessage> = stmt.query_map(rusqlite::params![case_id, e_like], |row| {
            Ok(EmailMessage {
                id: row.get(0)?, evidence_id: row.get(1)?, case_id: row.get(2)?, message_id: row.get(3)?,
                from_addr: row.get(4)?, from_display: row.get(5)?, to_addrs: row.get(6)?, cc_addrs: row.get(7)?,
                subject: row.get(8)?, date_sent: row.get(9)?, date_sent_utc: row.get(10)?,
                headers_raw: row.get(19)?,
                body_text: row.get(17)?,
                body_html: row.get(18)?,
                folder_name: row.get(11)?, folder_category: row.get(12)?,
                is_deleted: boolv(row, 13), deleted_recovered: boolv(row, 14), risk_score: u8v(row, 15), flags: row.get(16)?
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    };

    Ok(emails)
}

#[tauri::command]
pub async fn entity_heatmap(state: State<'_, AppState>, input: EntityInput) -> Result<Value, String> {
    let db = state.db.lock().await;
    let email_like = format!("%{}%", input.email_address);

    let points = if let Some(ref ev_id) = input.evidence_id.as_ref().filter(|s| !s.is_empty() && *s != "all") {
        let mut stmt = db.conn.prepare(
            "SELECT strftime('%Y-%m-%d', date_sent) as day, COUNT(*) as count 
             FROM emails WHERE case_id=?1 AND evidence_id=?2 AND (from_addr LIKE ?3 OR to_addrs LIKE ?3) AND day IS NOT NULL 
             GROUP BY day ORDER BY day ASC"
        ).map_err(|e| e.to_string())?;

        let res: Vec<Value> = stmt.query_map(rusqlite::params![&input.case_id, ev_id, email_like], |row| {
            Ok(serde_json::json!({
                "date": row.get::<_, String>(0)?,
                "count": row.get::<_, i64>(1)?
            }))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    } else {
        let mut stmt = db.conn.prepare(
            "SELECT strftime('%Y-%m-%d', date_sent) as day, COUNT(*) as count 
             FROM emails WHERE case_id=?1 AND (from_addr LIKE ?2 OR to_addrs LIKE ?2) AND day IS NOT NULL 
             GROUP BY day ORDER BY day ASC"
        ).map_err(|e| e.to_string())?;

        let res: Vec<Value> = stmt.query_map(rusqlite::params![&input.case_id, email_like], |row| {
            Ok(serde_json::json!({
                "date": row.get::<_, String>(0)?,
                "count": row.get::<_, i64>(1)?
            }))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    };

    Ok(serde_json::json!(points))
}
