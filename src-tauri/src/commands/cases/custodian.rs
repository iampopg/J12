pub fn is_automated_service(email: &str, display_name: Option<&str>, received_count: i64, sent_count: i64) -> bool {
    let email_lower = email.to_lowercase();
    let local = email_lower.split('@').next().unwrap_or("");
    let domain = email_lower.split('@').nth(1).unwrap_or("");

    if email_lower.is_empty() {
        return true;
    }

    let bot_prefixes = [
        "noreply", "no-reply", "no_reply", "donotreply", "do-not-reply",
        "newsletter", "news", "newsdigest", "marketing", "shop", "store",
        "orders", "order", "billing", "invoice", "notifications", "notification",
        "notify", "alerts", "alert", "updates", "update", "info", "support",
        "help", "service", "services", "mailer-daemon", "postmaster", "bounce",
        "bounces", "promo", "promotions", "promotion", "deals", "offers",
        "auto-confirm", "confirm", "system", "security", "feed", "digest",
        "jobs", "career", "careers", "invitations", "member", "rewards",
        "delivery", "bulk", "campaign", "survey", "donot-reply", "sales", "specials"
    ];

    for &p in &bot_prefixes {
        if local == p 
            || local.starts_with(&format!("{}.", p)) 
            || local.starts_with(&format!("{}-", p)) 
            || local.starts_with(&format!("{}_", p)) 
            || local.ends_with(&format!(".{}", p)) 
            || local.ends_with(&format!("-{}", p))
            || local.ends_with(&format!("_{}", p)) {
            return true;
        }
    }

    let bot_subdomains = [
        "emails.", "email.", "e-mail.", "em.", "news.", "marketing.", "mktg.",
        "bounce.", "bounces.", "delivery.", "bulk.", "notifications.", "alerts.",
        "mail.", "mailer.", "mailgun.", "sendgrid.", "mandrillapp.", "campaign.",
        "reply.", "newsletters.", "mailjet.", "exacttarget.", "mail1.", "mail2."
    ];

    for &sub in &bot_subdomains {
        if domain.starts_with(sub) || domain.contains(&format!(".{}", sub)) {
            return true;
        }
    }

    if received_count == 0 && sent_count >= 3 {
        if let Some(dname) = display_name {
            let dname_lower = dname.to_lowercase();
            let bot_keywords = [
                "sale", "rewards", "deals", "black friday", "cyber monday", "vip",
                "clearance", "discount", "digest", "news", "jobs", "alert", "alerts",
                "notification", "notifications", "special", "specials", "newsletter",
                "offers", "store", "shop", "promotions", "promo", "customer service",
                "support team", "automated", "mailer", "weekly", "daily", "inside apple",
                "glassdoor", "medium", "pet valu"
            ];
            for &k in &bot_keywords {
                if dname_lower.contains(k) {
                    return true;
                }
            }
        }
    }

    if received_count == 0 && sent_count >= 20 {
        if local.contains("mail") || local.contains("news") || local.contains("shop") || local.contains("alert") || local.contains("deal") {
            return true;
        }
    }

    false
}

pub fn detect_mailbox_custodian(
    conn: &rusqlite::Connection,
    case_id: &str,
    evidence_id: Option<&str>,
) -> Option<(String, Option<String>, String)> {
    let case_target: Option<(Option<String>, Option<String>)> = conn.query_row(
        "SELECT target_email, target_name FROM cases WHERE id = ?1",
        [case_id],
        |row| Ok((row.get(0)?, row.get(1)?))
    ).ok();

    if let Some((Some(t_email), t_name)) = case_target {
        let t_clean = t_email.trim().to_lowercase();
        if !t_clean.is_empty() {
            let exists: i64 = if let Some(ev_id) = evidence_id {
                conn.query_row(
                    "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND evidence_id = ?2 AND (lower(from_addr) LIKE ?3 OR lower(to_addrs) LIKE ?3)",
                    rusqlite::params![case_id, ev_id, format!("%{}%", t_clean)],
                    |r| r.get(0)
                ).unwrap_or(0)
            } else {
                conn.query_row(
                    "SELECT COUNT(*) FROM emails WHERE case_id = ?1 AND (lower(from_addr) LIKE ?2 OR lower(to_addrs) LIKE ?2)",
                    rusqlite::params![case_id, format!("%{}%", t_clean)],
                    |r| r.get(0)
                ).unwrap_or(0)
            };
            if exists > 0 {
                return Some((t_clean, t_name, "high (configured case target)".to_string()));
            }
        }
    }

    let sent_folder_sender: Result<(String, Option<String>, i64), _> = if let Some(ev_id) = evidence_id {
        conn.query_row(
            "SELECT from_addr, from_display, COUNT(*) as c FROM emails 
             WHERE case_id = ?1 AND evidence_id = ?2 
               AND (folder_category = 'sent' OR lower(folder_name) LIKE '%sent%' OR lower(folder_name) LIKE '%outbox%') 
               AND from_addr != '' 
             GROUP BY from_addr ORDER BY c DESC LIMIT 1",
            rusqlite::params![case_id, ev_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        )
    } else {
        conn.query_row(
            "SELECT from_addr, from_display, COUNT(*) as c FROM emails 
             WHERE case_id = ?1 
               AND (folder_category = 'sent' OR lower(folder_name) LIKE '%sent%' OR lower(folder_name) LIKE '%outbox%') 
               AND from_addr != '' 
             GROUP BY from_addr ORDER BY c DESC LIMIT 1",
            [case_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        )
    };

    if let Ok((email, name, count)) = sent_folder_sender {
        let clean = email.trim().to_lowercase();
        if !clean.is_empty() && count > 0 {
            return Some((clean, name, "high (sent folder owner)".to_string()));
        }
    }

    let to_addrs_rows: Vec<String> = if let Some(ev_id) = evidence_id {
        let mut stmt = conn.prepare(
            "SELECT to_addrs FROM emails WHERE case_id = ?1 AND evidence_id = ?2 AND to_addrs != '' AND to_addrs != '[]'"
        ).ok()?;
        let res = stmt.query_map(rusqlite::params![case_id, ev_id], |r| r.get(0)).ok()?.filter_map(|r| r.ok()).collect();
        res
    } else {
        let mut stmt = conn.prepare(
            "SELECT to_addrs FROM emails WHERE case_id = ?1 AND to_addrs != '' AND to_addrs != '[]'"
        ).ok()?;
        let res = stmt.query_map([case_id], |r| r.get(0)).ok()?.filter_map(|r| r.ok()).collect();
        res
    };

    let mut to_count_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for to_json in to_addrs_rows {
        if let Ok(list) = serde_json::from_str::<Vec<String>>(&to_json) {
            for addr in list {
                let clean = addr.trim().to_lowercase();
                if clean.contains('@') {
                    *to_count_map.entry(clean).or_insert(0) += 1;
                }
            }
        } else {
            let clean = to_json.trim_matches(|c| c == '[' || c == ']' || c == '"' || c == '\'').trim().to_lowercase();
            if clean.contains('@') {
                *to_count_map.entry(clean).or_insert(0) += 1;
            }
        }
    }

    let mut top_recipients: Vec<(String, i64)> = to_count_map.into_iter().collect();
    top_recipients.sort_by(|a, b| b.1.cmp(&a.1));

    for (recip_email, count) in top_recipients {
        let name_res: Option<String> = if let Some(ev_id) = evidence_id {
            conn.query_row(
                "SELECT from_display FROM emails WHERE case_id = ?1 AND evidence_id = ?2 AND lower(from_addr) = ?3 AND from_display IS NOT NULL AND from_display != '' LIMIT 1",
                rusqlite::params![case_id, ev_id, &recip_email],
                |r| r.get(0)
            ).ok()
        } else {
            conn.query_row(
                "SELECT from_display FROM emails WHERE case_id = ?1 AND lower(from_addr) = ?2 AND from_display IS NOT NULL AND from_display != '' LIMIT 1",
                rusqlite::params![case_id, &recip_email],
                |r| r.get(0)
            ).ok()
        };

        if !is_automated_service(&recip_email, name_res.as_deref(), count, 0) && count > 0 {
            return Some((recip_email, name_res, "high (inbox recipient custodian)".to_string()));
        }
    }

    None
}
