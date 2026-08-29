use serde::{Deserialize, Serialize};
use std::time::Instant;
use tauri::State;

use crate::AppState;
use super::super::helpers::{boolv, u8v};

#[derive(Debug, Deserialize)]
pub struct FtsSearchInput {
    pub case_id: String,
    pub query: String,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub evidence_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FtsSearchResultItem {
    pub id: String,
    pub evidence_id: String,
    pub case_id: String,
    pub message_id: Option<String>,
    pub from_addr: String,
    pub from_display: Option<String>,
    pub to_addrs: String,
    pub cc_addrs: String,
    pub subject: Option<String>,
    pub date_sent: Option<String>,
    pub date_sent_utc: Option<String>,
    pub folder_category: String,
    pub is_deleted: bool,
    pub deleted_recovered: bool,
    pub risk_score: u8,
    pub flags: Option<String>,
    pub snippet: Option<String>,
    pub match_rank: f64,
}

#[derive(Debug, Serialize)]
pub struct FtsSearchResponse {
    pub total_hits: usize,
    pub execution_ms: f64,
    pub query_parsed: String,
    pub items: Vec<FtsSearchResultItem>,
}

pub fn sanitize_fts5_query(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return "".to_string();
    }

    let has_boolean = trimmed.contains(" AND ") || trimmed.contains(" OR ") || trimmed.contains(" NOT ") || trimmed.starts_with("NEAR(");
    let has_quotes = trimmed.contains('"');

    if has_boolean || has_quotes {
        let quote_count = trimmed.chars().filter(|&c| c == '"').count();
        if quote_count % 2 != 0 {
            format!("{}\"", trimmed)
        } else {
            trimmed.to_string()
        }
    } else {
        let words: Vec<&str> = trimmed.split_whitespace().collect();
        if words.is_empty() {
            return "".to_string();
        }

        let formatted: Vec<String> = words.iter().map(|w| {
            let clean: String = w.chars().filter(|c| c.is_alphanumeric() || *c == '_' || *c == '@' || *c == '.' || *c == '-').collect();
            if clean.is_empty() {
                "".to_string()
            } else if clean.ends_with('*') {
                clean
            } else {
                format!("\"{}\"*", clean)
            }
        }).filter(|s| !s.is_empty()).collect();

        formatted.join(" ")
    }
}

#[tauri::command]
pub async fn fts_search(
    state: State<'_, AppState>,
    input: FtsSearchInput,
) -> Result<FtsSearchResponse, String> {
    let start_time = Instant::now();
    let db = state.db.lock().await;

    let fts_query = sanitize_fts5_query(&input.query);
    if fts_query.is_empty() {
        return Ok(FtsSearchResponse {
            total_hits: 0,
            execution_ms: 0.0,
            query_parsed: "".to_string(),
            items: vec![],
        });
    }

    let limit = input.limit.unwrap_or(100) as i64;
    let offset = input.offset.unwrap_or(0) as i64;

    let total_hits: usize = if let Some(ref ev_id) = input.evidence_id.as_ref().filter(|s| !s.is_empty() && *s != "all") {
        db.conn.query_row(
            "SELECT count(*) FROM emails_fts JOIN emails ON emails.id = emails_fts.email_id WHERE emails_fts.case_id = ?1 AND emails.evidence_id = ?2 AND emails_fts MATCH ?3",
            rusqlite::params![&input.case_id, ev_id, &fts_query],
            |r| r.get(0)
        ).unwrap_or(0)
    } else {
        db.conn.query_row(
            "SELECT count(*) FROM emails_fts WHERE case_id = ?1 AND emails_fts MATCH ?2",
            rusqlite::params![&input.case_id, &fts_query],
            |r| r.get(0)
        ).unwrap_or(0)
    };

    let mut items: Vec<FtsSearchResultItem> = Vec::new();

    if let Some(ref ev_id) = input.evidence_id.as_ref().filter(|s| !s.is_empty() && *s != "all") {
        let sql = format!(
            "SELECT emails.id, emails.evidence_id, emails.case_id, emails.message_id,
                    emails.from_addr, emails.from_display, emails.to_addrs, emails.cc_addrs,
                    emails.subject, emails.date_sent, emails.date_sent_utc, emails.folder_category,
                    emails.is_deleted, emails.deleted_recovered, emails.risk_score, emails.flags,
                    snippet(emails_fts, 5, '<mark class=\"fts-hit\">', '</mark>', '...', 28) as hit_snippet,
                    bm25(emails_fts) as rank_score
             FROM emails_fts
             JOIN emails ON emails.id = emails_fts.email_id
             WHERE emails_fts.case_id = ?1 AND emails.evidence_id = ?2 AND emails_fts MATCH ?3
             ORDER BY rank_score
             LIMIT {} OFFSET {}",
            limit, offset
        );
        let mut stmt = db.conn.prepare(&sql).map_err(|e| format!("FTS5 query syntax error: {}", e))?;
        let rows_res = stmt.query_map(rusqlite::params![&input.case_id, ev_id, &fts_query], |row| {
            Ok(FtsSearchResultItem {
                id: row.get(0)?,
                evidence_id: row.get(1)?,
                case_id: row.get(2)?,
                message_id: row.get(3)?,
                from_addr: row.get(4)?,
                from_display: row.get(5)?,
                to_addrs: row.get(6)?,
                cc_addrs: row.get(7)?,
                subject: row.get(8)?,
                date_sent: row.get(9)?,
                date_sent_utc: row.get(10)?,
                folder_category: row.get(11)?,
                is_deleted: boolv(row, 12),
                deleted_recovered: boolv(row, 13),
                risk_score: u8v(row, 14),
                flags: row.get(15)?,
                snippet: row.get(16)?,
                match_rank: row.get(17)?,
            })
        });

        if let Ok(rows) = rows_res {
            for r in rows.filter_map(|x| x.ok()) {
                items.push(r);
            }
        }
    } else {
        let sql = format!(
            "SELECT emails.id, emails.evidence_id, emails.case_id, emails.message_id,
                    emails.from_addr, emails.from_display, emails.to_addrs, emails.cc_addrs,
                    emails.subject, emails.date_sent, emails.date_sent_utc, emails.folder_category,
                    emails.is_deleted, emails.deleted_recovered, emails.risk_score, emails.flags,
                    snippet(emails_fts, 5, '<mark class=\"fts-hit\">', '</mark>', '...', 28) as hit_snippet,
                    bm25(emails_fts) as rank_score
             FROM emails_fts
             JOIN emails ON emails.id = emails_fts.email_id
             WHERE emails_fts.case_id = ?1 AND emails_fts MATCH ?2
             ORDER BY rank_score
             LIMIT {} OFFSET {}",
            limit, offset
        );
        let mut stmt = db.conn.prepare(&sql).map_err(|e| format!("FTS5 query syntax error: {}", e))?;
        let rows_res = stmt.query_map(rusqlite::params![&input.case_id, &fts_query], |row| {
            Ok(FtsSearchResultItem {
                id: row.get(0)?,
                evidence_id: row.get(1)?,
                case_id: row.get(2)?,
                message_id: row.get(3)?,
                from_addr: row.get(4)?,
                from_display: row.get(5)?,
                to_addrs: row.get(6)?,
                cc_addrs: row.get(7)?,
                subject: row.get(8)?,
                date_sent: row.get(9)?,
                date_sent_utc: row.get(10)?,
                folder_category: row.get(11)?,
                is_deleted: boolv(row, 12),
                deleted_recovered: boolv(row, 13),
                risk_score: u8v(row, 14),
                flags: row.get(15)?,
                snippet: row.get(16)?,
                match_rank: row.get(17)?,
            })
        });

        if let Ok(rows) = rows_res {
            for r in rows.filter_map(|x| x.ok()) {
                items.push(r);
            }
        }
    }

    let elapsed = start_time.elapsed().as_secs_f64() * 1000.0;

    crate::audit_logger::log_forensic_event(
        &input.case_id,
        "FTS_SEARCH",
        "INDEX_QUERY_EXECUTED",
        "Examiner",
        input.evidence_id.as_deref(),
        None,
        &format!("Executed SQLite FTS5 query [\"{}\"] -> {} hits in {:.2}ms", input.query, total_hits, elapsed)
    );

    Ok(FtsSearchResponse {
        total_hits,
        execution_ms: elapsed,
        query_parsed: fts_query,
        items,
    })
}
