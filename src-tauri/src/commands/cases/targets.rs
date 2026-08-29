use serde_json::Value;
use tauri::State;

use crate::AppState;
use super::custodian::{detect_mailbox_custodian, is_automated_service};

#[tauri::command]
pub async fn auto_detect_targets(state: State<'_, AppState>, input: Value) -> Result<Value, String> {
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

    let db = state.db.lock().await;

    let detected_custodian = detect_mailbox_custodian(&db.conn, &case_id, evidence_id.as_deref());

    let mut targets: Vec<Value> = Vec::new();
    let mut human_targets: Vec<Value> = Vec::new();
    let mut automated_targets: Vec<Value> = Vec::new();
    let mut seen_emails = std::collections::HashSet::new();

    if let Some((cust_email, cust_name, confidence_reason)) = detected_custodian {
        let (sent, recvd) = if let Some(ref ev_id) = evidence_id {
            let sent: i64 = db.conn.query_row(
                "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND evidence_id = ?2 AND lower(from_addr) = ?3",
                rusqlite::params![&case_id, ev_id, &cust_email],
                |r| r.get(0)
            ).unwrap_or(0);

            let recvd: i64 = db.conn.query_row(
                "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND evidence_id = ?2 AND (lower(to_addrs) LIKE ?3 OR lower(cc_addrs) LIKE ?3)",
                rusqlite::params![&case_id, ev_id, format!("%{}%", &cust_email)],
                |r| r.get(0)
            ).unwrap_or(0);
            (sent, recvd)
        } else {
            let sent: i64 = db.conn.query_row(
                "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND lower(from_addr) = ?2",
                rusqlite::params![&case_id, &cust_email],
                |r| r.get(0)
            ).unwrap_or(0);

            let recvd: i64 = db.conn.query_row(
                "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND (lower(to_addrs) LIKE ?2 OR lower(cc_addrs) LIKE ?2)",
                rusqlite::params![&case_id, format!("%{}%", &cust_email)],
                |r| r.get(0)
            ).unwrap_or(0);
            (sent, recvd)
        };

        seen_emails.insert(cust_email.to_lowercase());
        let org = cust_email.split('@').nth(1).unwrap_or("").to_string();
        let disp_name = cust_name.unwrap_or_else(|| cust_email.split('@').next().unwrap_or("").to_string());

        targets.push(serde_json::json!({
            "email": cust_email,
            "display_name": disp_name,
            "organization": org,
            "total_emails": sent + recvd,
            "sent": sent,
            "received": recvd,
            "confidence": "high",
            "is_primary_target": true,
            "is_custodian": true,
            "is_automated": false,
            "role": "Mailbox Custodian / Primary Target",
            "detection_note": confidence_reason
        }));
    }

    let sender_rows: Vec<(String, Option<String>, i64)> = if let Some(ref ev_id) = evidence_id {
        let mut stmt = db.conn.prepare(
            "SELECT from_addr, from_display, COUNT(*) as sent_count 
             FROM emails 
             WHERE case_id = ?1 AND evidence_id = ?2 AND from_addr != ''
             GROUP BY from_addr 
             ORDER BY sent_count DESC 
             LIMIT 50"
        ).map_err(|e| e.to_string())?;

        let res = stmt.query_map(rusqlite::params![&case_id, ev_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, i64>(2)?))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    } else {
        let mut stmt = db.conn.prepare(
            "SELECT from_addr, from_display, COUNT(*) as sent_count 
             FROM emails 
             WHERE case_id = ?1 AND from_addr != ''
             GROUP BY from_addr 
             ORDER BY sent_count DESC 
             LIMIT 50"
        ).map_err(|e| e.to_string())?;

        let res = stmt.query_map([&case_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, i64>(2)?))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    };

    for (email, name, sent_count) in sender_rows {
        let email_lower = email.trim().to_lowercase();
        if email_lower.is_empty() || seen_emails.contains(&email_lower) {
            continue;
        }
        seen_emails.insert(email_lower.clone());

        let org = email.split('@').nth(1).unwrap_or("").to_string();
        let recvd: i64 = if let Some(ref ev_id) = evidence_id {
            db.conn.query_row(
                "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND evidence_id = ?2 AND (lower(to_addrs) LIKE ?3 OR lower(cc_addrs) LIKE ?3)",
                rusqlite::params![&case_id, ev_id, format!("%{}%", &email_lower)],
                |r| r.get(0)
            ).unwrap_or(0)
        } else {
            db.conn.query_row(
                "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND (lower(to_addrs) LIKE ?2 OR lower(cc_addrs) LIKE ?2)",
                rusqlite::params![&case_id, format!("%{}%", &email_lower)],
                |r| r.get(0)
            ).unwrap_or(0)
        };

        let max_risk: i64 = if let Some(ref ev_id) = evidence_id {
            db.conn.query_row(
                "SELECT COALESCE(MAX(risk_score), 0) FROM emails WHERE case_id = ?1 AND evidence_id = ?2 AND (lower(from_addr) = ?3 OR lower(to_addrs) LIKE ?4)",
                rusqlite::params![&case_id, ev_id, &email_lower, format!("%{}%", &email_lower)],
                |r| r.get(0)
            ).unwrap_or(0)
        } else {
            db.conn.query_row(
                "SELECT COALESCE(MAX(risk_score), 0) FROM emails WHERE case_id = ?1 AND (lower(from_addr) = ?2 OR lower(to_addrs) LIKE ?3)",
                rusqlite::params![&case_id, &email_lower, format!("%{}%", &email_lower)],
                |r| r.get(0)
            ).unwrap_or(0)
        };

        let is_bot = is_automated_service(&email, name.as_deref(), recvd, sent_count);
        let total_emails = sent_count + recvd;
        let disp_name = name.unwrap_or_else(|| email.split('@').next().unwrap_or("").to_string());

        let candidate_obj = serde_json::json!({
            "email": email,
            "display_name": disp_name,
            "organization": org,
            "total_emails": total_emails,
            "sent": sent_count,
            "received": recvd,
            "max_risk_score": max_risk,
            "confidence": if is_bot { "automated_service" } else if sent_count > 0 && recvd > 0 { "high" } else if total_emails > 10 { "medium" } else { "low" },
            "is_primary_target": false,
            "is_custodian": false,
            "is_automated": is_bot,
            "role": if is_bot { "Automated Service / Bot" } else if sent_count > 0 && recvd > 0 { "Interactive Correspondent" } else { "Person of Interest" }
        });

        if is_bot {
            automated_targets.push(candidate_obj);
        } else {
            human_targets.push(candidate_obj);
        }
    }

    human_targets.sort_by(|a, b| {
        let a_interactive = a["sent"].as_i64().unwrap_or(0) > 0 && a["received"].as_i64().unwrap_or(0) > 0;
        let b_interactive = b["sent"].as_i64().unwrap_or(0) > 0 && b["received"].as_i64().unwrap_or(0) > 0;
        if a_interactive != b_interactive {
            return b_interactive.cmp(&a_interactive);
        }
        let a_risk = a["max_risk_score"].as_i64().unwrap_or(0);
        let b_risk = b["max_risk_score"].as_i64().unwrap_or(0);
        if a_risk != b_risk {
            return b_risk.cmp(&a_risk);
        }
        let a_tot = a["total_emails"].as_i64().unwrap_or(0);
        let b_tot = b["total_emails"].as_i64().unwrap_or(0);
        b_tot.cmp(&a_tot)
    });

    automated_targets.sort_by(|a, b| {
        let a_tot = a["total_emails"].as_i64().unwrap_or(0);
        let b_tot = b["total_emails"].as_i64().unwrap_or(0);
        b_tot.cmp(&a_tot)
    });

    targets.extend(human_targets);
    targets.extend(automated_targets);

    let total_entities: i64 = if let Some(ref ev_id) = evidence_id {
        db.conn.query_row(
            "SELECT COUNT(DISTINCT from_addr) FROM emails WHERE case_id = ?1 AND evidence_id = ?2 AND from_addr != ''",
            rusqlite::params![&case_id, ev_id],
            |r| r.get(0)
        ).unwrap_or(0)
    } else {
        db.conn.query_row(
            "SELECT COUNT(DISTINCT from_addr) FROM emails WHERE case_id = ?1 AND from_addr != ''",
            [&case_id],
            |r| r.get(0)
        ).unwrap_or(0)
    };

    Ok(serde_json::json!({
        "targets": targets.clone(),
        "candidates": targets,
        "total_case_entities": total_entities
    }))
}
