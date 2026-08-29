use std::collections::{HashMap, HashSet};
use serde_json::Value;
use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn timeline_data(state: State<'_, AppState>, input: Value) -> Result<Value, String> {
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

    let sql = if evidence_id.is_some() {
        "SELECT strftime('%Y-%m-%d', COALESCE(date_sent_utc, date_sent)) as day, 
                COUNT(*) as total,
                SUM(CASE WHEN folder_category = 'sent' THEN 1 ELSE 0 END) as sent,
                SUM(CASE WHEN folder_category != 'sent' THEN 1 ELSE 0 END) as received,
                SUM(CASE WHEN risk_score > 50 THEN 1 ELSE 0 END) as high_risk,
                SUM(CASE WHEN is_deleted = 1 OR deleted_recovered = 1 THEN 1 ELSE 0 END) as deleted
         FROM emails 
         WHERE case_id = ?1 AND evidence_id = ?2 AND day IS NOT NULL AND day != ''
         GROUP BY day 
         ORDER BY day ASC"
    } else {
        "SELECT strftime('%Y-%m-%d', COALESCE(date_sent_utc, date_sent)) as day, 
                COUNT(*) as total,
                SUM(CASE WHEN folder_category = 'sent' THEN 1 ELSE 0 END) as sent,
                SUM(CASE WHEN folder_category != 'sent' THEN 1 ELSE 0 END) as received,
                SUM(CASE WHEN risk_score > 50 THEN 1 ELSE 0 END) as high_risk,
                SUM(CASE WHEN is_deleted = 1 OR deleted_recovered = 1 THEN 1 ELSE 0 END) as deleted
         FROM emails 
         WHERE case_id = ?1 AND day IS NOT NULL AND day != ''
         GROUP BY day 
         ORDER BY day ASC"
    };

    let mut stmt = db.conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows: Vec<Value> = if let Some(ref ev_id) = evidence_id {
        stmt.query_map(rusqlite::params![&case_id, ev_id], |row| {
            Ok(serde_json::json!({
                "date": row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                "total": row.get::<_, i64>(1).unwrap_or(0),
                "sent": row.get::<_, i64>(2).unwrap_or(0),
                "received": row.get::<_, i64>(3).unwrap_or(0),
                "high_risk": row.get::<_, i64>(4).unwrap_or(0),
                "deleted": row.get::<_, i64>(5).unwrap_or(0)
            }))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect()
    } else {
        stmt.query_map([&case_id], |row| {
            Ok(serde_json::json!({
                "date": row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                "total": row.get::<_, i64>(1).unwrap_or(0),
                "sent": row.get::<_, i64>(2).unwrap_or(0),
                "received": row.get::<_, i64>(3).unwrap_or(0),
                "high_risk": row.get::<_, i64>(4).unwrap_or(0),
                "deleted": row.get::<_, i64>(5).unwrap_or(0)
            }))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect()
    };

    let min_date = rows.first().and_then(|r| r["date"].as_str()).unwrap_or("").to_string();
    let max_date = rows.last().and_then(|r| r["date"].as_str()).unwrap_or("").to_string();

    Ok(serde_json::json!({
        "daily": rows,
        "date_range": {
            "min": min_date,
            "max": max_date
        }
    }))
}

#[tauri::command]
pub async fn graph_data(state: State<'_, AppState>, input: Value) -> Result<Value, String> {
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

    let rows = if let Some(ref ev_id) = evidence_id {
        let mut stmt = db.conn.prepare(
            "SELECT from_addr, to_addrs, risk_score FROM emails WHERE case_id = ?1 AND evidence_id = ?2 AND from_addr != '' LIMIT 5000"
        ).map_err(|e| e.to_string())?;

        let res: Vec<(String, String, u8)> = stmt.query_map(rusqlite::params![&case_id, ev_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<i64>>(2)?.unwrap_or(0) as u8
            ))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    } else {
        let mut stmt = db.conn.prepare(
            "SELECT from_addr, to_addrs, risk_score FROM emails WHERE case_id = ?1 AND from_addr != '' LIMIT 5000"
        ).map_err(|e| e.to_string())?;

        let res: Vec<(String, String, u8)> = stmt.query_map([&case_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<i64>>(2)?.unwrap_or(0) as u8
            ))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        res
    };

    let mut node_set: HashSet<String> = HashSet::new();
    let mut node_risks: HashMap<String, u8> = HashMap::new();
    let mut node_sent: HashMap<String, u32> = HashMap::new();
    let mut node_recv: HashMap<String, u32> = HashMap::new();
    let mut edge_map: HashMap<(String, String), u32> = HashMap::new();

    for (from, to_json, risk) in rows {
        node_set.insert(from.clone());
        *node_sent.entry(from.clone()).or_insert(0) += 1;
        let current_risk = node_risks.entry(from.clone()).or_insert(0);
        if risk > *current_risk {
            *current_risk = risk;
        }

        let recipients: Vec<String> = serde_json::from_str(&to_json).unwrap_or_default();
        for r in recipients {
            if !r.is_empty() {
                node_set.insert(r.clone());
                *node_recv.entry(r.clone()).or_insert(0) += 1;
                let r_risk = node_risks.entry(r.clone()).or_insert(0);
                if risk > *r_risk {
                    *r_risk = risk;
                }

                let edge_key = if from < r {
                    (from.clone(), r.clone())
                } else {
                    (r.clone(), from.clone())
                };

                *edge_map.entry(edge_key).or_insert(0) += 1;
            }
        }
    }

    let nodes: Vec<Value> = node_set.into_iter().map(|email| {
        let risk = node_risks.get(&email).cloned().unwrap_or(0);
        let s = node_sent.get(&email).cloned().unwrap_or(0);
        let r = node_recv.get(&email).cloned().unwrap_or(0);
        serde_json::json!({
            "id": email,
            "label": email,
            "name": email,
            "risk_score": risk,
            "sent": s,
            "received": r,
            "total": s + r,
            "is_target": false
        })
    }).collect();

    let edges: Vec<Value> = edge_map.into_iter().map(|((source, target), weight)| {
        serde_json::json!({
            "source": source,
            "target": target,
            "weight": weight
        })
    }).collect();

    Ok(serde_json::json!({
        "nodes": nodes,
        "edges": edges
    }))
}
