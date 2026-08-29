use std::collections::HashSet;
use crate::db::generate_id;
use super::types::ForensicTaxonomyArtifact;
use super::signatures::CompiledRegexes;
use super::validators::{luhn_check, validate_routing_number, validate_btc_base58, validate_solana_base58, validate_tron_base58, validate_iban, validate_swift_bic};

pub fn scan_financial_and_wallets(
    artifacts: &mut Vec<ForensicTaxonomyArtifact>,
    seen: &mut HashSet<String>,
    re: &CompiledRegexes,
    eid: &str,
    from_addr: &str,
    subj_opt: &Option<String>,
    date_opt: &Option<String>,
    from_lower: &str,
    subj_lower: &str,
    full_text: &str,
    full_text_lower: &str,
    subj: &str,
) {
    for cap in re.cc_spaced.captures_iter(full_text) {
        let cc_raw = cap[1].replace([' ', '-'], "");
        if luhn_check(&cc_raw) {
            let key = format!("cc:{}", cc_raw);
            if seen.insert(key) {
                let card_type = if cc_raw.starts_with('4') { "Visa" } else if cc_raw.starts_with("34") || cc_raw.starts_with("37") { "Amex" } else if cc_raw.starts_with("6011") { "Discover" } else { "MasterCard" };
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "financial".to_string(),
                    subcategory_id: "credit_cards".to_string(),
                    title: format!("Credit Card ({})", card_type),
                    primary_value: cap[1].to_string(),
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("Luhn-Verified Credit Card Number ({})", card_type),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.to_string(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.to_string(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }
    }

    for cap in re.cc_raw.captures_iter(full_text) {
        let cc_raw = cap[1].to_string();
        if luhn_check(&cc_raw) {
            let key = format!("cc:{}", cc_raw);
            if seen.insert(key) {
                let card_type = if cc_raw.starts_with('4') { "Visa" } else if cc_raw.starts_with("34") || cc_raw.starts_with("37") { "Amex" } else if cc_raw.starts_with("6011") { "Discover" } else { "MasterCard" };
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "financial".to_string(),
                    subcategory_id: "credit_cards".to_string(),
                    title: format!("Credit Card ({})", card_type),
                    primary_value: cc_raw.clone(),
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("Luhn-Verified Card Number: {}", cc_raw),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.to_string(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.to_string(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }
    }

    if full_text_lower.contains("routing") || full_text_lower.contains("aba") {
        for cap in re.routing.captures_iter(full_text) {
            let r_no = cap[1].trim().to_string();
            if validate_routing_number(&r_no) {
                let key = format!("routing:{}", r_no);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "financial".to_string(),
                        subcategory_id: "routing_numbers".to_string(),
                        title: "US ABA Bank Routing Number".to_string(),
                        primary_value: format!("Routing #: {}", r_no),
                        secondary_value: Some(from_addr.to_string()),
                        details: format!("Verified US 9-digit ABA Bank Routing Number: {}", r_no),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.to_string(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.to_string(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }
    }

    if full_text_lower.contains("iban") {
        for cap in re.iban.captures_iter(full_text) {
            let iban = cap[1].trim().to_string();
            if validate_iban(&iban) {
                let key = format!("iban:{}", iban);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "financial".to_string(),
                        subcategory_id: "iban".to_string(),
                        title: "IBAN Bank Account Number".to_string(),
                        primary_value: format!("IBAN: {}", iban),
                        secondary_value: Some(from_addr.to_string()),
                        details: format!("Verified ISO 7064 Mod 97-10 IBAN: {}", iban),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.to_string(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.to_string(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }
    }

    if full_text_lower.contains("swift") || full_text_lower.contains("bic") {
        for cap in re.swift.captures_iter(full_text) {
            let swift = cap[1].trim().to_string();
            if validate_swift_bic(&swift) {
                let key = format!("swift:{}", swift);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "financial".to_string(),
                        subcategory_id: "swift_bic".to_string(),
                        title: "SWIFT / BIC Bank Code".to_string(),
                        primary_value: format!("SWIFT: {}", swift),
                        secondary_value: Some(from_addr.to_string()),
                        details: format!("Verified ISO 9362 SWIFT/BIC Bank Identifier: {}", swift),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.to_string(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.to_string(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }
    }

    if full_text_lower.contains("account") || full_text_lower.contains("acct") {
        for cap in re.account.captures_iter(full_text) {
            let acc = cap[1].trim().to_string();
            if acc.len() >= 8 && acc.len() <= 17 && !acc.chars().all(|c| c == acc.chars().next().unwrap_or('0')) && !acc.starts_with("2024") && !acc.starts_with("2025") && !acc.starts_with("2026") {
                let key = format!("acct:{}", acc);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "financial".to_string(),
                        subcategory_id: "account_numbers".to_string(),
                        title: "Bank Account Number".to_string(),
                        primary_value: format!("Account #: {}", acc),
                        secondary_value: Some(from_addr.to_string()),
                        details: format!("Extracted financial account number: {}", acc),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.to_string(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.to_string(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }
    }

    if full_text_lower.contains("sort code") || full_text_lower.contains("sort-code") || full_text_lower.contains("sortcode") {
        for cap in re.sort_code.captures_iter(full_text) {
            let sort = cap[1].trim().to_string();
            let key = format!("sort:{}", sort);
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "financial".to_string(),
                    subcategory_id: "sort_code".to_string(),
                    title: "UK / Ireland Bank Sort Code".to_string(),
                    primary_value: format!("Sort Code: {}", sort),
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("Bank clearing sort code: {}", sort),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.to_string(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.to_string(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }
    }

    if subj_lower.contains("statement") || subj_lower.contains("estatement") || full_text_lower.contains("electronic statement") || full_text_lower.contains("bank statement") {
        let bank_name = if from_lower.contains("huntington") || subj_lower.contains("huntington") {
            "Huntington Bank"
        } else if from_lower.contains("chase") || subj_lower.contains("chase") {
            "Chase Bank"
        } else if from_lower.contains("simmons") || subj_lower.contains("simmons") {
            "Simmons Bank"
        } else if from_lower.contains("afbank") || subj_lower.contains("afbank") {
            "Armed Forces Bank"
        } else if from_lower.contains("bankofamerica") || from_lower.contains("bofa") {
            "Bank of America"
        } else if from_lower.contains("wellsfargo") {
            "Wells Fargo"
        } else {
            "Commercial Bank / Financial Institution"
        };

        let key = format!("statement:{}:{}", eid, bank_name);
        if seen.insert(key) {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "fintech_banking".to_string(),
                subcategory_id: "bank_statements".to_string(),
                title: format!("{} - Financial Statement Notification", bank_name),
                primary_value: subj.to_string(),
                secondary_value: Some(from_addr.to_string()),
                details: format!("Electronic bank statement notification from {} ({})", bank_name, from_addr),
                severity: "critical".to_string(),
                artifact_type: "native".to_string(),
                confidence: Some("high".to_string()),
                email_id: eid.to_string(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.to_string(),
                date_sent_utc: date_opt.clone(),
            });
        }
    }

    // Wallets & crypto
    if full_text_lower.contains("btc") || full_text_lower.contains("bitcoin") || full_text_lower.contains("crypto") || full_text.contains('1') || full_text.contains('3') {
        for cap in re.btc_legacy.captures_iter(full_text) {
            let btc = cap[1].to_string();
            if validate_btc_base58(&btc) {
                let key = format!("btc:{}", btc);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "bitcoin_p2pkh".to_string(),
                        title: "Bitcoin Legacy (P2PKH) Address".to_string(),
                        primary_value: btc.clone(),
                        secondary_value: Some(from_addr.to_string()),
                        details: format!("Verified Bitcoin Base58 address: {}", btc),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.to_string(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.to_string(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }
    }

    if full_text.contains("bc1") {
        for cap in re.btc_bech32.captures_iter(full_text) {
            let btc = cap[1].to_string();
            let key = format!("btc_bech:{}", btc);
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "crypto".to_string(),
                    subcategory_id: "bitcoin_bech32".to_string(),
                    title: "Bitcoin SegWit (Bech32) Address".to_string(),
                    primary_value: btc.clone(),
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("Bitcoin SegWit Native Bech32 Address: {}", btc),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.to_string(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.to_string(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }
    }

    if full_text.contains("0x") {
        for cap in re.eth.captures_iter(full_text) {
            let eth = cap[1].to_string();
            if eth != "0x0000000000000000000000000000000000000000" {
                let key = format!("eth:{}", eth);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "ethereum".to_string(),
                        title: "Ethereum / ERC-20 Wallet Address".to_string(),
                        primary_value: eth.clone(),
                        secondary_value: Some(from_addr.to_string()),
                        details: format!("Ethereum / EVM Address: {}", eth),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.to_string(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.to_string(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }
    }

    if full_text_lower.contains("tron") || full_text_lower.contains("trx") || full_text_lower.contains("trc20") || full_text_lower.contains("trc-20") || full_text_lower.contains("usdt-trc") {
        for cap in re.tron.captures_iter(full_text) {
            let trx = cap[1].to_string();
            if validate_tron_base58(&trx) {
                let key = format!("trx:{}", trx);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "tron".to_string(),
                        title: "TRON (TRX / USDT-TRC20) Address".to_string(),
                        primary_value: trx.clone(),
                        secondary_value: Some(from_addr.to_string()),
                        details: format!("Verified TRON Network Address: {}", trx),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.to_string(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.to_string(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }
    }

    if full_text_lower.contains("solana") || full_text_lower.contains("phantom") || full_text_lower.contains("solflare") || full_text_lower.contains("sol:") || full_text_lower.contains(" sol ") {
        for cap in re.sol.captures_iter(full_text) {
            let sol = cap[1].to_string();
            if validate_solana_base58(&sol) {
                let key = format!("sol:{}", sol);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "solana".to_string(),
                        title: "Solana (SOL) Wallet Address".to_string(),
                        primary_value: sol.clone(),
                        secondary_value: Some(from_addr.to_string()),
                        details: format!("Verified Solana 32-byte Ed25519 Public Key: {}", sol),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.to_string(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.to_string(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }
    }

    if full_text_lower.contains("litecoin") || full_text_lower.contains(" ltc ") || full_text_lower.contains("ltc:") {
        for cap in re.ltc.captures_iter(full_text) {
            let ltc = cap[1].to_string();
            if validate_btc_base58(&ltc) {
                let key = format!("ltc:{}", ltc);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "litecoin".to_string(),
                        title: "Litecoin (LTC) Address".to_string(),
                        primary_value: ltc.clone(),
                        secondary_value: Some(from_addr.to_string()),
                        details: format!("Verified Litecoin Network Address: {}", ltc),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.to_string(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.to_string(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }
    }

    if full_text_lower.contains("dogecoin") || full_text_lower.contains(" doge ") || full_text_lower.contains("doge:") {
        for cap in re.doge.captures_iter(full_text) {
            let doge = cap[1].to_string();
            if validate_btc_base58(&doge) {
                let key = format!("doge:{}", doge);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "dogecoin".to_string(),
                        title: "Dogecoin (DOGE) Address".to_string(),
                        primary_value: doge.clone(),
                        secondary_value: Some(from_addr.to_string()),
                        details: format!("Verified Dogecoin Network Address: {}", doge),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.to_string(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.to_string(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }
    }

    if full_text_lower.contains("monero") || full_text_lower.contains(" xmr ") || full_text_lower.contains("xmr:") {
        for cap in re.xmr.captures_iter(full_text) {
            let xmr = cap[1].to_string();
            let key = format!("xmr:{}", xmr);
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "crypto".to_string(),
                    subcategory_id: "monero".to_string(),
                    title: "Monero (XMR) Privacy Address".to_string(),
                    primary_value: xmr.clone(),
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("Monero (XMR) Stealth Address: {}", xmr),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.to_string(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.to_string(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }
    }

    if full_text_lower.contains("bitcoin:") || full_text_lower.contains("ethereum:") || full_text_lower.contains("solana:") {
        for cap in re.crypto_uri.captures_iter(full_text) {
            let uri = cap[1].to_string();
            let key = format!("crypto_uri:{}", uri);
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "crypto".to_string(),
                    subcategory_id: "qr_wallet_uris".to_string(),
                    title: "Cryptocurrency Wallet Payment URI".to_string(),
                    primary_value: uri.clone(),
                    secondary_value: Some(from_addr.to_string()),
                    details: format!("Payment URI schema: {}", uri),
                    severity: "high".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.to_string(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.to_string(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }
    }
}
