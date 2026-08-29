use serde_json::Value;
use tauri::State;

use crate::AppState;
use super::custodian::{detect_mailbox_custodian, is_automated_service};

#[tauri::command]
pub async fn target_profile(state: State<'_, AppState>, input: Value) -> Result<Value, String> {
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
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s != "all");

    let specified_email = input["target_email"].as_str()
        .or_else(|| input["targetEmail"].as_str())
        .or_else(|| input["input"]["target_email"].as_str())
        .or_else(|| input["input"]["targetEmail"].as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let db = state.db.lock().await;

    let case_row = db.conn.query_row(
        "SELECT title, case_number, target_email, target_name, target_organization FROM cases WHERE id = ?1",
        [&case_id],
        |row| Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            row.get::<_, Option<String>>(3)?.unwrap_or_default(),
            row.get::<_, Option<String>>(4)?.unwrap_or_default()
        ))
    );

    let (case_title, case_number, case_target_email, case_target_name, case_target_org) = match case_row {
        Ok((t, num, e, n, o)) => (t, num, e, n, o),
        Err(_) => ("Investigation".to_string(), "J12-001".to_string(), String::new(), String::new(), String::new()),
    };

    let mut target_email = specified_email.clone().unwrap_or(case_target_email);
    let mut target_name = case_target_name;
    let mut target_org = case_target_org;

    if !target_email.is_empty() && evidence_id.is_some() {
        let ev_id = evidence_id.as_ref().unwrap();
        let target_like = format!("%{}%", target_email);
        let exists: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND evidence_id = ?2 AND (lower(from_addr) LIKE ?3 OR lower(to_addrs) LIKE ?3)",
            rusqlite::params![&case_id, ev_id, &target_like.to_lowercase()],
            |r| r.get(0)
        ).unwrap_or(0);

        if exists == 0 {
            target_email = String::new();
            target_name = String::new();
            target_org = String::new();
        }
    }

    if target_email.is_empty() {
        if let Some((cust_email, cust_name, _)) = detect_mailbox_custodian(&db.conn, &case_id, evidence_id.as_deref()) {
            target_email = cust_email.clone();
            target_name = cust_name.unwrap_or_else(|| cust_email.split('@').next().unwrap_or("").to_string());
            target_org = cust_email.split('@').nth(1).unwrap_or("").to_string();
        }
    }

    if target_email.is_empty() {
        return Ok(serde_json::json!({
            "case_id": case_id,
            "case_title": case_title,
            "case_number": case_number,
            "target_email": null,
            "target_name": null,
            "target_organization": null,
            "sent_count": 0,
            "received_count": 0,
            "total_emails": 0,
            "first_seen": null,
            "last_seen": null,
            "top_correspondents": [],
            "top_subjects": [],
            "display_names": [],
            "x_mailers": [],
            "originating_ips": [],
            "risk_score": 0,
            "flagged_count": 0,
            "attachment_count": 0,
            "recent_communications": []
        }));
    }

    let target_lower = target_email.to_lowercase();
    let target_like = format!("%{}%", target_lower);

    let sent_count: i64 = if let Some(ref ev_id) = evidence_id {
        db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND evidence_id = ?2 AND lower(from_addr) LIKE ?3",
            rusqlite::params![&case_id, ev_id, &target_like],
            |r| r.get(0)
        ).unwrap_or(0)
    } else {
        db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND lower(from_addr) LIKE ?2",
            rusqlite::params![&case_id, &target_like],
            |r| r.get(0)
        ).unwrap_or(0)
    };

    let received_count: i64 = if let Some(ref ev_id) = evidence_id {
        db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND evidence_id = ?2 AND (lower(to_addrs) LIKE ?3 OR lower(cc_addrs) LIKE ?3 OR lower(bcc_addrs) LIKE ?3)",
            rusqlite::params![&case_id, ev_id, &target_like],
            |r| r.get(0)
        ).unwrap_or(0)
    } else {
        db.conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND (lower(to_addrs) LIKE ?2 OR lower(cc_addrs) LIKE ?2 OR lower(bcc_addrs) LIKE ?2)",
            rusqlite::params![&case_id, &target_like],
            |r| r.get(0)
        ).unwrap_or(0)
    };

    let total_emails = sent_count + received_count;

    let (first_seen, last_seen): (Option<String>, Option<String>) = if let Some(ref ev_id) = evidence_id {
        db.conn.query_row(
            "SELECT MIN(date_sent_utc), MAX(date_sent_utc) FROM emails WHERE case_id = ?1 AND evidence_id = ?2 AND (lower(from_addr) LIKE ?3 OR lower(to_addrs) LIKE ?3)",
            rusqlite::params![&case_id, ev_id, &target_like],
            |r| Ok((r.get(0)?, r.get(1)?))
        ).unwrap_or((None, None))
    } else {
        db.conn.query_row(
            "SELECT MIN(date_sent_utc), MAX(date_sent_utc) FROM emails WHERE case_id = ?1 AND (lower(from_addr) LIKE ?2 OR lower(to_addrs) LIKE ?2)",
            rusqlite::params![&case_id, &target_like],
            |r| Ok((r.get(0)?, r.get(1)?))
        ).unwrap_or((None, None))
    };

    let (risk_score, flagged_count): (i64, i64) = if let Some(ref ev_id) = evidence_id {
        db.conn.query_row(
            "SELECT COALESCE(MAX(risk_score), 0), COUNT(CASE WHEN risk_score > 25 THEN 1 END) 
             FROM emails WHERE case_id = ?1 AND evidence_id = ?2 AND (lower(from_addr) LIKE ?3 OR lower(to_addrs) LIKE ?3)",
            rusqlite::params![&case_id, ev_id, &target_like],
            |r| Ok((r.get(0)?, r.get(1)?))
        ).unwrap_or((0, 0))
    } else {
        db.conn.query_row(
            "SELECT COALESCE(MAX(risk_score), 0), COUNT(CASE WHEN risk_score > 25 THEN 1 END) 
             FROM emails WHERE case_id = ?1 AND (lower(from_addr) LIKE ?2 OR lower(to_addrs) LIKE ?2)",
            rusqlite::params![&case_id, &target_like],
            |r| Ok((r.get(0)?, r.get(1)?))
        ).unwrap_or((0, 0))
    };

    let attachment_count: i64 = if let Some(ref ev_id) = evidence_id {
        db.conn.query_row(
            "SELECT COUNT(*) FROM attachments a JOIN emails e ON a.email_id = e.id 
             WHERE e.case_id = ?1 AND e.evidence_id = ?2 AND (lower(e.from_addr) LIKE ?3 OR lower(e.to_addrs) LIKE ?3)",
            rusqlite::params![&case_id, ev_id, &target_like],
            |r| r.get(0)
        ).unwrap_or(0)
    } else {
        db.conn.query_row(
            "SELECT COUNT(*) FROM attachments a JOIN emails e ON a.email_id = e.id 
             WHERE e.case_id = ?1 AND (lower(e.from_addr) LIKE ?2 OR lower(e.to_addrs) LIKE ?2)",
            rusqlite::params![&case_id, &target_like],
            |r| r.get(0)
        ).unwrap_or(0)
    };

    let display_names: Vec<String> = if let Some(ref ev_id) = evidence_id {
        let mut name_stmt = db.conn.prepare(
            "SELECT DISTINCT from_display FROM emails 
             WHERE case_id = ?1 AND evidence_id = ?2 AND lower(from_addr) LIKE ?3 AND from_display IS NOT NULL AND from_display != '' 
             LIMIT 10"
        ).map_err(|e| e.to_string())?;
        let res: Vec<String> = name_stmt.query_map(rusqlite::params![&case_id, ev_id, &target_like], |r| r.get(0))
            .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    } else {
        let mut name_stmt = db.conn.prepare(
            "SELECT DISTINCT from_display FROM emails 
             WHERE case_id = ?1 AND lower(from_addr) LIKE ?2 AND from_display IS NOT NULL AND from_display != '' 
             LIMIT 10"
        ).map_err(|e| e.to_string())?;
        let res: Vec<String> = name_stmt.query_map(rusqlite::params![&case_id, &target_like], |r| r.get(0))
            .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    };

    let x_mailers: Vec<String> = if let Some(ref ev_id) = evidence_id {
        let mut mailer_stmt = db.conn.prepare(
            "SELECT DISTINCT x_mailer FROM emails 
             WHERE case_id = ?1 AND evidence_id = ?2 AND lower(from_addr) LIKE ?3 AND x_mailer IS NOT NULL AND x_mailer != '' 
             LIMIT 6"
        ).map_err(|e| e.to_string())?;
        let res: Vec<String> = mailer_stmt.query_map(rusqlite::params![&case_id, ev_id, &target_like], |r| r.get(0))
            .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    } else {
        let mut mailer_stmt = db.conn.prepare(
            "SELECT DISTINCT x_mailer FROM emails 
             WHERE case_id = ?1 AND lower(from_addr) LIKE ?2 AND x_mailer IS NOT NULL AND x_mailer != '' 
             LIMIT 6"
        ).map_err(|e| e.to_string())?;
        let res: Vec<String> = mailer_stmt.query_map(rusqlite::params![&case_id, &target_like], |r| r.get(0))
            .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    };

    let originating_ips: Vec<String> = if let Some(ref ev_id) = evidence_id {
        let mut ip_stmt = db.conn.prepare(
            "SELECT DISTINCT x_originating_ip FROM emails 
             WHERE case_id = ?1 AND evidence_id = ?2 AND lower(from_addr) LIKE ?3 AND x_originating_ip IS NOT NULL AND x_originating_ip != '' 
             LIMIT 6"
        ).map_err(|e| e.to_string())?;
        let res: Vec<String> = ip_stmt.query_map(rusqlite::params![&case_id, ev_id, &target_like], |r| r.get(0))
            .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    } else {
        let mut ip_stmt = db.conn.prepare(
            "SELECT DISTINCT x_originating_ip FROM emails 
             WHERE case_id = ?1 AND lower(from_addr) LIKE ?2 AND x_originating_ip IS NOT NULL AND x_originating_ip != '' 
             LIMIT 6"
        ).map_err(|e| e.to_string())?;
        let res: Vec<String> = ip_stmt.query_map(rusqlite::params![&case_id, &target_like], |r| r.get(0))
            .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    };

    let mut corr_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

    let peer_senders: Vec<(String, i64)> = if let Some(ref ev_id) = evidence_id {
        let mut corr_stmt = db.conn.prepare(
            "SELECT from_addr, COUNT(*) as cnt FROM emails 
             WHERE case_id = ?1 AND evidence_id = ?2 AND lower(to_addrs) LIKE ?3 AND lower(from_addr) NOT LIKE ?3 AND from_addr != '' 
             GROUP BY from_addr ORDER BY cnt DESC LIMIT 15"
        ).map_err(|e| e.to_string())?;
        let res: Vec<(String, i64)> = corr_stmt.query_map(rusqlite::params![&case_id, ev_id, &target_like], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    } else {
        let mut corr_stmt = db.conn.prepare(
            "SELECT from_addr, COUNT(*) as cnt FROM emails 
             WHERE case_id = ?1 AND lower(to_addrs) LIKE ?2 AND lower(from_addr) NOT LIKE ?2 AND from_addr != '' 
             GROUP BY from_addr ORDER BY cnt DESC LIMIT 15"
        ).map_err(|e| e.to_string())?;
        let res: Vec<(String, i64)> = corr_stmt.query_map(rusqlite::params![&case_id, &target_like], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    };

    for (sender, cnt) in peer_senders {
        let clean = sender.trim().to_lowercase();
        if !clean.is_empty() && !clean.contains(&target_lower) {
            *corr_map.entry(clean).or_insert(0) += cnt;
        }
    }

    let peer_recips_raw: Vec<String> = if let Some(ref ev_id) = evidence_id {
        let mut stmt = db.conn.prepare(
            "SELECT to_addrs FROM emails WHERE case_id = ?1 AND evidence_id = ?2 AND lower(from_addr) LIKE ?3 AND to_addrs != '' AND to_addrs != '[]'"
        ).map_err(|e| e.to_string())?;
        let res: Vec<String> = stmt.query_map(rusqlite::params![&case_id, ev_id, &target_like], |r| r.get(0))
            .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    } else {
        let mut stmt = db.conn.prepare(
            "SELECT to_addrs FROM emails WHERE case_id = ?1 AND lower(from_addr) LIKE ?2 AND to_addrs != '' AND to_addrs != '[]'"
        ).map_err(|e| e.to_string())?;
        let res: Vec<String> = stmt.query_map(rusqlite::params![&case_id, &target_like], |r| r.get(0))
            .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    };

    for to_json in peer_recips_raw {
        if let Ok(list) = serde_json::from_str::<Vec<String>>(&to_json) {
            for addr in list {
                let clean = addr.trim().to_lowercase();
                if !clean.is_empty() && !clean.contains(&target_lower) && clean.contains('@') {
                    *corr_map.entry(clean).or_insert(0) += 1;
                }
            }
        } else {
            let clean = to_json.trim_matches(|c| c == '[' || c == ']' || c == '"' || c == '\'').trim().to_lowercase();
            if !clean.is_empty() && !clean.contains(&target_lower) && clean.contains('@') {
                *corr_map.entry(clean).or_insert(0) += 1;
            }
        }
    }

    let mut top_correspondents: Vec<(String, i64)> = corr_map.into_iter().collect();
    top_correspondents.sort_by(|a, b| b.1.cmp(&a.1));
    top_correspondents.truncate(10);

    let top_subjects: Vec<(String, i64)> = if let Some(ref ev_id) = evidence_id {
        let mut subj_stmt = db.conn.prepare(
            "SELECT subject, COUNT(*) as cnt FROM emails 
             WHERE case_id = ?1 AND evidence_id = ?2 AND (lower(from_addr) LIKE ?3 OR lower(to_addrs) LIKE ?3) AND subject IS NOT NULL AND subject != ''
             GROUP BY subject ORDER BY cnt DESC LIMIT 8"
        ).map_err(|e| e.to_string())?;
        let res: Vec<(String, i64)> = subj_stmt.query_map(rusqlite::params![&case_id, ev_id, &target_like], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    } else {
        let mut subj_stmt = db.conn.prepare(
            "SELECT subject, COUNT(*) as cnt FROM emails 
             WHERE case_id = ?1 AND (lower(from_addr) LIKE ?2 OR lower(to_addrs) LIKE ?2) AND subject IS NOT NULL AND subject != ''
             GROUP BY subject ORDER BY cnt DESC LIMIT 8"
        ).map_err(|e| e.to_string())?;
        let res: Vec<(String, i64)> = subj_stmt.query_map(rusqlite::params![&case_id, &target_like], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    };

    let recent_communications: Vec<Value> = if let Some(ref ev_id) = evidence_id {
        let mut comm_stmt = db.conn.prepare(
            "SELECT id, subject, date_sent_utc, from_addr, to_addrs, risk_score 
             FROM emails 
             WHERE case_id = ?1 AND evidence_id = ?2 AND (lower(from_addr) LIKE ?3 OR lower(to_addrs) LIKE ?3) 
             ORDER BY date_sent_utc DESC LIMIT 8"
        ).map_err(|e| e.to_string())?;
        let res: Vec<Value> = comm_stmt.query_map(rusqlite::params![&case_id, ev_id, &target_like], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "subject": row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "(No Subject)".to_string()),
                "date": row.get::<_, Option<String>>(2)?,
                "from": row.get::<_, String>(3)?,
                "to": row.get::<_, String>(4)?,
                "risk_score": row.get::<_, i64>(5)?
            }))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    } else {
        let mut comm_stmt = db.conn.prepare(
            "SELECT id, subject, date_sent_utc, from_addr, to_addrs, risk_score 
             FROM emails 
             WHERE case_id = ?1 AND (lower(from_addr) LIKE ?2 OR lower(to_addrs) LIKE ?2) 
             ORDER BY date_sent_utc DESC LIMIT 8"
        ).map_err(|e| e.to_string())?;
        let res: Vec<Value> = comm_stmt.query_map(rusqlite::params![&case_id, &target_like], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "subject": row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "(No Subject)".to_string()),
                "date": row.get::<_, Option<String>>(2)?,
                "from": row.get::<_, String>(3)?,
                "to": row.get::<_, String>(4)?,
                "risk_score": row.get::<_, i64>(5)?
            }))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    };

    let is_bot = is_automated_service(&target_email, Some(&target_name), received_count, sent_count);

    Ok(serde_json::json!({
        "case_id": case_id,
        "case_title": case_title,
        "case_number": case_number,
        "target_email": target_email,
        "target_name": if target_name.is_empty() { target_email.split('@').next().unwrap_or("").to_string() } else { target_name },
        "target_organization": if target_org.is_empty() { target_email.split('@').nth(1).unwrap_or("").to_string() } else { target_org },
        "sent_count": sent_count,
        "received_count": received_count,
        "total_emails": total_emails,
        "first_seen": first_seen,
        "last_seen": last_seen,
        "top_correspondents": top_correspondents,
        "top_subjects": top_subjects,
        "display_names": display_names,
        "x_mailers": x_mailers,
        "originating_ips": originating_ips,
        "risk_score": risk_score,
        "flagged_count": flagged_count,
        "attachment_count": attachment_count,
        "recent_communications": recent_communications,
        "is_automated": is_bot,
        "role": if is_bot { "Automated Service / Bot" } else if sent_count > 0 && received_count > 0 { "Mailbox Custodian / Person of Interest" } else { "Person of Interest" }
    }))
}
