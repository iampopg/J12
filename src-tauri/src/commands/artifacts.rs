use std::collections::{BTreeMap, HashSet};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use crate::AppState;
use crate::db::generate_id;
use super::attachments::classify_attachment_category;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxonomySubcategorySummary {
    pub subcategory_id: String,
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxonomyDomainSummary {
    pub domain_id: String,
    pub name: String,
    pub icon: String,
    pub total_count: usize,
    pub subcategories: Vec<TaxonomySubcategorySummary>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForensicTaxonomyArtifact {
    pub id: String,
    pub domain_id: String,
    pub subcategory_id: String,
    pub title: String,
    pub primary_value: String,
    pub secondary_value: Option<String>,
    pub details: String,
    pub severity: String,
    pub artifact_type: String,
    pub confidence: Option<String>,
    pub email_id: String,
    pub email_subject: Option<String>,
    pub email_from: String,
    pub date_sent_utc: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// HIGH-PRECISION FALSE POSITIVE (FP) REDUCTION VALIDATORS (>80% TRUE POSITIVES)
// ─────────────────────────────────────────────────────────────────────────────

/// Luhn algorithm for validating credit card numbers
pub fn luhn_check(num_str: &str) -> bool {
    let digits: Vec<u32> = num_str.chars().filter_map(|c| c.to_digit(10)).collect();
    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }
    if digits.iter().all(|&d| d == digits[0]) {
        return false;
    }
    let mut sum = 0;
    let mut double = false;
    for &d in digits.iter().rev() {
        let val = if double {
            let doubled = d * 2;
            if doubled > 9 { doubled - 9 } else { doubled }
        } else {
            d
        };
        sum += val;
        double = !double;
    }
    sum % 10 == 0
}

/// US 9-Digit ABA Bank Routing Checksum Validator
pub fn validate_routing_number(num_str: &str) -> bool {
    let digits: Vec<u32> = num_str.chars().filter_map(|c| c.to_digit(10)).collect();
    if digits.len() != 9 || digits.iter().all(|&d| d == digits[0]) {
        return false;
    }
    let sum = 3 * (digits[0] + digits[3] + digits[6])
            + 7 * (digits[1] + digits[4] + digits[7])
            + 1 * (digits[2] + digits[5] + digits[8]);
    sum % 10 == 0
}

/// US Social Security Number (SSN) Structure Validator
pub fn validate_ssn(ssn_str: &str) -> bool {
    let clean: String = ssn_str.chars().filter(|c| c.is_ascii_digit()).collect();
    if clean.len() != 9 || clean.chars().all(|c| c == clean.chars().next().unwrap()) {
        return false;
    }
    let area: u32 = clean[0..3].parse().unwrap_or(0);
    let group: u32 = clean[3..5].parse().unwrap_or(0);
    let serial: u32 = clean[5..9].parse().unwrap_or(0);
    if area == 0 || area == 666 || area >= 900 || group == 0 || serial == 0 {
        return false;
    }
    if clean == "123456789" || clean == "987654321" {
        return false;
    }
    true
}

/// Base58 Bitcoin Address Character & Cryptographic Checksum Validator
pub fn validate_btc_base58(addr: &str) -> bool {
    if addr.len() < 26 || addr.len() > 35 { return false; }
    if !addr.starts_with('1') && !addr.starts_with('3') { return false; }
    let alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut decoded = [0u8; 35];
    let mut decoded_len = 0;
    
    for c in addr.chars() {
        let mut carry = match alphabet.find(c) {
            Some(idx) => idx as u32,
            None => return false,
        };
        for i in 0..decoded_len {
            carry += (decoded[i] as u32) * 58;
            decoded[i] = (carry & 0xFF) as u8;
            carry >>= 8;
        }
        while carry > 0 {
            if decoded_len >= 35 { return false; }
            decoded[decoded_len] = (carry & 0xFF) as u8;
            decoded_len += 1;
            carry >>= 8;
        }
    }
    for c in addr.chars() {
        if c == '1' {
            if decoded_len >= 35 { return false; }
            decoded[decoded_len] = 0;
            decoded_len += 1;
        } else {
            break;
        }
    }
    if decoded_len != 25 { return false; }
    decoded[0..decoded_len].reverse();
    
    // Verify 4-byte double SHA-256 checksum
    use sha2::{Sha256, Digest};
    let mut hasher1 = Sha256::new();
    hasher1.update(&decoded[0..21]);
    let hash1 = hasher1.finalize();
    
    let mut hasher2 = Sha256::new();
    hasher2.update(&hash1);
    let hash2 = hasher2.finalize();
    
    &hash2[0..4] == &decoded[21..25]
}

/// Phone Number Sanitizer & Quality Check
pub fn validate_phone(p: &str) -> bool {
    let digits: Vec<char> = p.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 10 || digits.len() > 15 { return false; }
    if digits.iter().all(|&c| c == digits[0]) { return false; }
    if (p.starts_with("19") || p.starts_with("20")) && p.contains('-') && digits.len() <= 8 {
        return false;
    }
    true
}

// ─────────────────────────────────────────────────────────────────────────────
// EXHAUSTIVE DIGITAL FOOTPRINT & APP / SERVICE KNOWLEDGE BASE
// ─────────────────────────────────────────────────────────────────────────────
struct AppSignature {
    name: &'static str,
    domain_id: &'static str,
    subcategory: &'static str,
    keywords: &'static [&'static str],
    category_title: &'static str,
}

static APP_SIGNATURES: &[AppSignature] = &[
    // 🌐 SOCIAL MEDIA & COMMUNITIES
    AppSignature { name: "Snapchat", domain_id: "social_media", subcategory: "snapchat", keywords: &["snapchat.com", "sc-corp.com", "team snapchat"], category_title: "Social Media (Snapchat)" },
    AppSignature { name: "Twitter / X", domain_id: "social_media", subcategory: "twitter_x", keywords: &["twitter.com", "x.com", "api.twitter.com", "twittermail.com"], category_title: "Social Network (Twitter/X)" },
    AppSignature { name: "Instagram", domain_id: "social_media", subcategory: "instagram", keywords: &["instagram.com", "instagr.am", "mail.instagram.com"], category_title: "Social Network (Instagram)" },
    AppSignature { name: "Facebook", domain_id: "social_media", subcategory: "facebook", keywords: &["facebook.com", "fb.com", "facebookmail.com"], category_title: "Social Network (Facebook)" },
    AppSignature { name: "TikTok", domain_id: "social_media", subcategory: "tiktok", keywords: &["tiktok.com", "byteoversea.com", "tiktokmail.com"], category_title: "Social Video (TikTok)" },
    AppSignature { name: "LinkedIn", domain_id: "social_media", subcategory: "linkedin", keywords: &["linkedin.com", "linkedinmail.com"], category_title: "Professional Network (LinkedIn)" },
    AppSignature { name: "Reddit", domain_id: "social_media", subcategory: "reddit", keywords: &["reddit.com", "redditmail.com"], category_title: "Social Community (Reddit)" },
    AppSignature { name: "Pinterest", domain_id: "social_media", subcategory: "pinterest", keywords: &["pinterest.com", "pinterestmail.com"], category_title: "Social Platform (Pinterest)" },
    AppSignature { name: "YouTube", domain_id: "social_media", subcategory: "youtube", keywords: &["youtube.com", "youtu.be"], category_title: "Video Platform (YouTube)" },
    AppSignature { name: "Twitch", domain_id: "social_media", subcategory: "twitch", keywords: &["twitch.tv"], category_title: "Live Streaming (Twitch)" },
    AppSignature { name: "Tumblr", domain_id: "social_media", subcategory: "tumblr", keywords: &["tumblr.com"], category_title: "Social Blog (Tumblr)" },
    AppSignature { name: "Threads", domain_id: "social_media", subcategory: "threads", keywords: &["threads.net"], category_title: "Social Network (Threads)" },
    AppSignature { name: "Bluesky", domain_id: "social_media", subcategory: "bluesky", keywords: &["bsky.app", "bsky.social"], category_title: "Social Network (Bluesky)" },
    AppSignature { name: "VKontakte", domain_id: "social_media", subcategory: "vk", keywords: &["vk.com", "vkontakte.ru"], category_title: "Social Network (VKontakte)" },

    // 💬 MESSAGING & ENCRYPTED CHAT
    AppSignature { name: "Telegram", domain_id: "messaging_apps", subcategory: "telegram", keywords: &["telegram.org", "t.me", "telegram messenger"], category_title: "Encrypted Messenger (Telegram)" },
    AppSignature { name: "Signal", domain_id: "messaging_apps", subcategory: "signal", keywords: &["signal.org", "signal.me", "signal messenger"], category_title: "Private Messenger (Signal)" },
    AppSignature { name: "WhatsApp", domain_id: "messaging_apps", subcategory: "whatsapp", keywords: &["whatsapp.com", "wa.me", "whatsapp business"], category_title: "Messaging App (WhatsApp)" },
    AppSignature { name: "Discord", domain_id: "messaging_apps", subcategory: "discord", keywords: &["discord.com", "discord.gg", "discordapp.com"], category_title: "Chat Platform (Discord)" },
    AppSignature { name: "Session", domain_id: "messaging_apps", subcategory: "session", keywords: &["getsession.org", "session messenger"], category_title: "Anonymous Chat (Session)" },
    AppSignature { name: "Threema", domain_id: "messaging_apps", subcategory: "threema", keywords: &["threema.ch", "threema id"], category_title: "Secure Messenger (Threema)" },
    AppSignature { name: "Wickr", domain_id: "messaging_apps", subcategory: "wickr", keywords: &["wickr.com", "wickr me"], category_title: "Ephemeral Chat (Wickr)" },
    AppSignature { name: "Element / Matrix", domain_id: "messaging_apps", subcategory: "matrix", keywords: &["element.io", "matrix.org"], category_title: "Matrix Protocol (Element)" },
    AppSignature { name: "Viber", domain_id: "messaging_apps", subcategory: "viber", keywords: &["viber.com"], category_title: "Messaging App (Viber)" },
    AppSignature { name: "WeChat", domain_id: "messaging_apps", subcategory: "wechat", keywords: &["wechat.com", "weixin.qq.com"], category_title: "Messaging App (WeChat)" },
    AppSignature { name: "Line", domain_id: "messaging_apps", subcategory: "line", keywords: &["line.me", "naver.jp"], category_title: "Messaging App (Line)" },
    AppSignature { name: "Skype", domain_id: "messaging_apps", subcategory: "skype", keywords: &["skype.com", "skype.net"], category_title: "VoIP & Chat (Skype)" },
    AppSignature { name: "Kik", domain_id: "messaging_apps", subcategory: "kik", keywords: &["kik.com"], category_title: "Messaging App (Kik)" },

    // 🪙 CRYPTO PLATFORMS, EXCHANGES & WEB3
    AppSignature { name: "Binance", domain_id: "crypto_platforms", subcategory: "binance", keywords: &["binance.com", "binance exchange", "binance verification"], category_title: "Crypto Exchange (Binance)" },
    AppSignature { name: "Coinbase", domain_id: "crypto_platforms", subcategory: "coinbase", keywords: &["coinbase.com", "coinbase pro", "coinbase commerce"], category_title: "Crypto Platform (Coinbase)" },
    AppSignature { name: "Kraken", domain_id: "crypto_platforms", subcategory: "kraken", keywords: &["kraken.com", "kraken exchange"], category_title: "Crypto Exchange (Kraken)" },
    AppSignature { name: "KuCoin", domain_id: "crypto_platforms", subcategory: "kucoin", keywords: &["kucoin.com", "kucoin exchange"], category_title: "Crypto Exchange (KuCoin)" },
    AppSignature { name: "MetaMask", domain_id: "crypto_platforms", subcategory: "metamask", keywords: &["metamask.io", "metamask wallet"], category_title: "Crypto Wallet (MetaMask)" },
    AppSignature { name: "Trust Wallet", domain_id: "crypto_platforms", subcategory: "trust_wallet", keywords: &["trustwallet.com", "trust wallet"], category_title: "Crypto Wallet (Trust Wallet)" },
    AppSignature { name: "Ledger", domain_id: "crypto_platforms", subcategory: "ledger", keywords: &["ledger.com", "ledger live"], category_title: "Hardware Wallet (Ledger)" },
    AppSignature { name: "Trezor", domain_id: "crypto_platforms", subcategory: "trezor", keywords: &["trezor.io", "trezor suite"], category_title: "Hardware Wallet (Trezor)" },
    AppSignature { name: "Bybit", domain_id: "crypto_platforms", subcategory: "bybit", keywords: &["bybit.com", "bybit exchange"], category_title: "Crypto Exchange (Bybit)" },
    AppSignature { name: "OKX", domain_id: "crypto_platforms", subcategory: "okx", keywords: &["okx.com", "okex.com"], category_title: "Crypto Exchange (OKX)" },
    AppSignature { name: "Bitfinex", domain_id: "crypto_platforms", subcategory: "bitfinex", keywords: &["bitfinex.com"], category_title: "Crypto Exchange (Bitfinex)" },
    AppSignature { name: "Uniswap", domain_id: "crypto_platforms", subcategory: "uniswap", keywords: &["uniswap.org"], category_title: "DeFi DEX (Uniswap)" },
    AppSignature { name: "Phantom", domain_id: "crypto_platforms", subcategory: "phantom", keywords: &["phantom.app", "phantom wallet"], category_title: "Solana Wallet (Phantom)" },
    AppSignature { name: "Exodus", domain_id: "crypto_platforms", subcategory: "exodus", keywords: &["exodus.com", "exodus wallet"], category_title: "Crypto Wallet (Exodus)" },
    AppSignature { name: "Paxful", domain_id: "crypto_platforms", subcategory: "paxful", keywords: &["paxful.com", "paxful p2p"], category_title: "P2P Crypto (Paxful)" },
    AppSignature { name: "Gemini", domain_id: "crypto_platforms", subcategory: "gemini", keywords: &["gemini.com", "gemini exchange"], category_title: "Crypto Exchange (Gemini)" },
    AppSignature { name: "OpenSea", domain_id: "crypto_platforms", subcategory: "opensea", keywords: &["opensea.io"], category_title: "NFT Marketplace (OpenSea)" },
    AppSignature { name: "BitMEX", domain_id: "crypto_platforms", subcategory: "bitmex", keywords: &["bitmex.com"], category_title: "Crypto Derivatives (BitMEX)" },

    // ❤️ DATING & ROMANCE PLATFORMS
    AppSignature { name: "Tinder", domain_id: "dating_apps", subcategory: "tinder", keywords: &["gotinder.com", "tinder.com"], category_title: "Dating App (Tinder)" },
    AppSignature { name: "Bumble", domain_id: "dating_apps", subcategory: "bumble", keywords: &["bumble.com"], category_title: "Dating App (Bumble)" },
    AppSignature { name: "Hinge", domain_id: "dating_apps", subcategory: "hinge", keywords: &["hinge.co"], category_title: "Dating App (Hinge)" },
    AppSignature { name: "Badoo", domain_id: "dating_apps", subcategory: "badoo", keywords: &["badoo.com"], category_title: "Dating App (Badoo)" },
    AppSignature { name: "Grindr", domain_id: "dating_apps", subcategory: "grindr", keywords: &["grindr.com"], category_title: "Dating App (Grindr)" },
    AppSignature { name: "Match.com", domain_id: "dating_apps", subcategory: "match", keywords: &["match.com"], category_title: "Dating Service (Match.com)" },
    AppSignature { name: "OkCupid", domain_id: "dating_apps", subcategory: "okcupid", keywords: &["okcupid.com"], category_title: "Dating App (OkCupid)" },
    AppSignature { name: "Ashley Madison", domain_id: "dating_apps", subcategory: "ashley_madison", keywords: &["ashleymadison.com"], category_title: "Discreet Dating (Ashley Madison)" },
    AppSignature { name: "OnlyFans", domain_id: "dating_apps", subcategory: "onlyfans", keywords: &["onlyfans.com"], category_title: "Subscription Platform (OnlyFans)" },
    AppSignature { name: "Fansly", domain_id: "dating_apps", subcategory: "fansly", keywords: &["fansly.com"], category_title: "Subscription Platform (Fansly)" },

    // 🏦 FINTECH, COMMERCIAL BANKS & PAYMENTS
    AppSignature { name: "Huntington Bank", domain_id: "fintech_banking", subcategory: "huntington", keywords: &["huntington.com", "email.huntington.com", "huntingtononline"], category_title: "Commercial Bank (Huntington Bank)" },
    AppSignature { name: "Chase Bank / JPMorgan", domain_id: "fintech_banking", subcategory: "chase", keywords: &["chase.com", "emailonline.chase.com", "chaseonline"], category_title: "Commercial Bank (Chase)" },
    AppSignature { name: "Simmons Bank", domain_id: "fintech_banking", subcategory: "simmons", keywords: &["simmonsbank.com"], category_title: "Commercial Bank (Simmons Bank)" },
    AppSignature { name: "Armed Forces Bank", domain_id: "fintech_banking", subcategory: "afbank", keywords: &["afbank.com"], category_title: "Military Bank (Armed Forces Bank)" },
    AppSignature { name: "Bank of America", domain_id: "fintech_banking", subcategory: "bofa", keywords: &["bankofamerica.com", "bofa.com"], category_title: "Commercial Bank (Bank of America)" },
    AppSignature { name: "Wells Fargo", domain_id: "fintech_banking", subcategory: "wells_fargo", keywords: &["wellsfargo.com"], category_title: "Commercial Bank (Wells Fargo)" },
    AppSignature { name: "Citibank", domain_id: "fintech_banking", subcategory: "citi", keywords: &["citi.com", "citibank.com"], category_title: "Commercial Bank (Citibank)" },
    AppSignature { name: "Capital One", domain_id: "fintech_banking", subcategory: "capital_one", keywords: &["capitalone.com"], category_title: "Commercial Bank (Capital One)" },
    AppSignature { name: "PNC Bank", domain_id: "fintech_banking", subcategory: "pnc", keywords: &["pnc.com"], category_title: "Commercial Bank (PNC Bank)" },
    AppSignature { name: "ApexPay", domain_id: "fintech_banking", subcategory: "apexpay", keywords: &["apexpay.org"], category_title: "Payment Platform (ApexPay)" },
    AppSignature { name: "Paybis Crypto", domain_id: "crypto_platforms", subcategory: "paybis", keywords: &["paybis.com"], category_title: "Crypto Exchange (Paybis)" },
    AppSignature { name: "Zypto Crypto", domain_id: "crypto_platforms", subcategory: "zypto", keywords: &["zypto.com"], category_title: "Crypto Wallet & Pay (Zypto)" },
    AppSignature { name: "CoinCodex", domain_id: "crypto_platforms", subcategory: "coincodex", keywords: &["coincodex.com"], category_title: "Crypto Platform (CoinCodex)" },
    AppSignature { name: "TaxAct / W-2 Taxes", domain_id: "financial", subcategory: "tax_filing", keywords: &["taxact.com", "turbotax.com", "hrblock.com"], category_title: "Tax Filing & IRS (TaxAct)" },
    AppSignature { name: "PayPal", domain_id: "fintech_banking", subcategory: "paypal", keywords: &["paypal.com", "service@paypal"], category_title: "Payment Service (PayPal)" },
    AppSignature { name: "Stripe", domain_id: "fintech_banking", subcategory: "stripe", keywords: &["stripe.com"], category_title: "Payment Gateway (Stripe)" },
    AppSignature { name: "Venmo", domain_id: "fintech_banking", subcategory: "venmo", keywords: &["venmo.com"], category_title: "P2P Payments (Venmo)" },
    AppSignature { name: "Cash App", domain_id: "fintech_banking", subcategory: "cash_app", keywords: &["cash.app", "square.com"], category_title: "P2P Payments (Cash App)" },
    AppSignature { name: "Revolut", domain_id: "fintech_banking", subcategory: "revolut", keywords: &["revolut.com"], category_title: "Digital Bank (Revolut)" },
    AppSignature { name: "Wise", domain_id: "fintech_banking", subcategory: "wise", keywords: &["wise.com", "transferwise.com"], category_title: "Cross-Border Payments (Wise)" },
    AppSignature { name: "Payoneer", domain_id: "fintech_banking", subcategory: "payoneer", keywords: &["payoneer.com"], category_title: "Payment Platform (Payoneer)" },
    AppSignature { name: "Zelle", domain_id: "fintech_banking", subcategory: "zelle", keywords: &["zellepay.com"], category_title: "Bank Network (Zelle)" },
    AppSignature { name: "Robinhood", domain_id: "fintech_banking", subcategory: "robinhood", keywords: &["robinhood.com"], category_title: "Trading App (Robinhood)" },
    AppSignature { name: "eToro", domain_id: "fintech_banking", subcategory: "etoro", keywords: &["etoro.com"], category_title: "Trading Platform (eToro)" },
    AppSignature { name: "Monzo", domain_id: "fintech_banking", subcategory: "monzo", keywords: &["monzo.com"], category_title: "Digital Bank (Monzo)" },
    AppSignature { name: "N26", domain_id: "fintech_banking", subcategory: "n26", keywords: &["n26.com"], category_title: "Digital Bank (N26)" },
    AppSignature { name: "Klarna", domain_id: "fintech_banking", subcategory: "klarna", keywords: &["klarna.com"], category_title: "BNPL Fintech (Klarna)" },
    AppSignature { name: "Western Union", domain_id: "fintech_banking", subcategory: "western_union", keywords: &["westernunion.com"], category_title: "Money Transfer (Western Union)" },

    // 📱 MOBILE & ON-DEMAND APPS
    AppSignature { name: "Uber", domain_id: "mobile_apps", subcategory: "uber", keywords: &["uber.com", "ubereats.com"], category_title: "Ride & Delivery (Uber)" },
    AppSignature { name: "Lyft", domain_id: "mobile_apps", subcategory: "lyft", keywords: &["lyft.com", "lyftmail.com"], category_title: "Rideshare App (Lyft)" },
    AppSignature { name: "DoorDash", domain_id: "mobile_apps", subcategory: "doordash", keywords: &["doordash.com"], category_title: "Food Delivery (DoorDash)" },
    AppSignature { name: "Deliveroo", domain_id: "mobile_apps", subcategory: "deliveroo", keywords: &["deliveroo.co.uk", "deliveroo.com"], category_title: "Food Delivery (Deliveroo)" },
    AppSignature { name: "Instacart", domain_id: "mobile_apps", subcategory: "instacart", keywords: &["instacart.com"], category_title: "Grocery Delivery (Instacart)" },
    AppSignature { name: "Airbnb", domain_id: "mobile_apps", subcategory: "airbnb", keywords: &["airbnb.com", "airbnbmail.com"], category_title: "Lodging Booking (Airbnb)" },
    AppSignature { name: "Booking.com", domain_id: "mobile_apps", subcategory: "booking", keywords: &["booking.com"], category_title: "Travel Booking (Booking.com)" },
    AppSignature { name: "Spotify", domain_id: "mobile_apps", subcategory: "spotify", keywords: &["spotify.com"], category_title: "Music Streaming (Spotify)" },
    AppSignature { name: "Netflix", domain_id: "mobile_apps", subcategory: "netflix", keywords: &["netflix.com"], category_title: "Video Streaming (Netflix)" },
    AppSignature { name: "Disney+", domain_id: "mobile_apps", subcategory: "disney", keywords: &["disneyplus.com", "disneystreaming.com"], category_title: "Streaming (Disney+)" },
    AppSignature { name: "Apple Services", domain_id: "mobile_apps", subcategory: "apple", keywords: &["apple.com", "itunes.com", "icloud.com"], category_title: "Apple Ecosystem (iOS/iCloud)" },
    AppSignature { name: "Google Play", domain_id: "mobile_apps", subcategory: "google_play", keywords: &["play.google.com"], category_title: "App Store (Google Play)" },
    AppSignature { name: "Duolingo", domain_id: "mobile_apps", subcategory: "duolingo", keywords: &["duolingo.com"], category_title: "Language App (Duolingo)" },
    AppSignature { name: "Strava", domain_id: "mobile_apps", subcategory: "strava", keywords: &["strava.com"], category_title: "Fitness GPS (Strava)" },

    // 🛍️ E-COMMERCE & MARKETPLACES
    AppSignature { name: "Amazon", domain_id: "ecommerce_shopping", subcategory: "amazon", keywords: &["amazon.com", "auto-confirm@amazon", "amazon.co.uk", "amazon.de"], category_title: "Marketplace (Amazon)" },
    AppSignature { name: "eBay", domain_id: "ecommerce_shopping", subcategory: "ebay", keywords: &["ebay.com", "ebay.co.uk", "ebay@ebay"], category_title: "Auction & Marketplace (eBay)" },
    AppSignature { name: "AliExpress", domain_id: "ecommerce_shopping", subcategory: "aliexpress", keywords: &["aliexpress.com", "alibaba.com"], category_title: "Marketplace (AliExpress)" },
    AppSignature { name: "Temu", domain_id: "ecommerce_shopping", subcategory: "temu", keywords: &["temu.com", "temu-mail.com"], category_title: "Marketplace (Temu)" },
    AppSignature { name: "Shein", domain_id: "ecommerce_shopping", subcategory: "shein", keywords: &["shein.com"], category_title: "Fast Fashion (Shein)" },
    AppSignature { name: "Etsy", domain_id: "ecommerce_shopping", subcategory: "etsy", keywords: &["etsy.com"], category_title: "Craft Marketplace (Etsy)" },
    AppSignature { name: "Walmart", domain_id: "ecommerce_shopping", subcategory: "walmart", keywords: &["walmart.com"], category_title: "Retail (Walmart)" },
    AppSignature { name: "Vinted", domain_id: "ecommerce_shopping", subcategory: "vinted", keywords: &["vinted.com", "vinted.fr"], category_title: "Resale App (Vinted)" },
    AppSignature { name: "StockX", domain_id: "ecommerce_shopping", subcategory: "stockx", keywords: &["stockx.com"], category_title: "Sneaker/Goods (StockX)" },

    // 🤖 AI, CLOUD & DEVELOPER PLATFORMS
    AppSignature { name: "OpenAI / ChatGPT", domain_id: "ai_cloud_dev", subcategory: "openai", keywords: &["openai.com", "chatgpt.com"], category_title: "AI Platform (OpenAI/ChatGPT)" },
    AppSignature { name: "Anthropic Claude", domain_id: "ai_cloud_dev", subcategory: "anthropic", keywords: &["anthropic.com", "claude.ai"], category_title: "AI Platform (Anthropic/Claude)" },
    AppSignature { name: "Midjourney", domain_id: "ai_cloud_dev", subcategory: "midjourney", keywords: &["midjourney.com"], category_title: "Generative AI (Midjourney)" },
    AppSignature { name: "GitHub", domain_id: "ai_cloud_dev", subcategory: "github", keywords: &["github.com", "notifications@github.com"], category_title: "Code Repository (GitHub)" },
    AppSignature { name: "GitLab", domain_id: "ai_cloud_dev", subcategory: "gitlab", keywords: &["gitlab.com"], category_title: "Code Repository (GitLab)" },
    AppSignature { name: "AWS Cloud", domain_id: "ai_cloud_dev", subcategory: "aws", keywords: &["amazonaws.com", "aws.amazon.com"], category_title: "Cloud Services (AWS)" },
    AppSignature { name: "Google Cloud", domain_id: "ai_cloud_dev", subcategory: "gcp", keywords: &["cloud.google.com"], category_title: "Cloud Services (GCP)" },
    AppSignature { name: "Vercel", domain_id: "ai_cloud_dev", subcategory: "vercel", keywords: &["vercel.com"], category_title: "Cloud Deployment (Vercel)" },
    AppSignature { name: "Cloudflare", domain_id: "ai_cloud_dev", subcategory: "cloudflare", keywords: &["cloudflare.com"], category_title: "Infrastructure (Cloudflare)" },

    // 🛡️ VPNS, ENCRYPTED MAIL & PRIVACY
    AppSignature { name: "ProtonMail", domain_id: "vpns_privacy", subcategory: "proton", keywords: &["proton.me", "protonmail.com", "protonvpn.com"], category_title: "Encrypted Mail (Proton)" },
    AppSignature { name: "Tutanota / Tuta", domain_id: "vpns_privacy", subcategory: "tuta", keywords: &["tuta.com", "tutanota.com"], category_title: "Encrypted Mail (Tuta)" },
    AppSignature { name: "SimpleLogin", domain_id: "vpns_privacy", subcategory: "simplelogin", keywords: &["simplelogin.io", "simplelogin.co"], category_title: "Email Alias (SimpleLogin)" },
    AppSignature { name: "DuckDuckGo", domain_id: "vpns_privacy", subcategory: "duckduckgo", keywords: &["duck.com", "duckduckgo.com"], category_title: "Privacy Relay (DuckDuckGo)" },
    AppSignature { name: "NordVPN", domain_id: "vpns_privacy", subcategory: "nordvpn", keywords: &["nordvpn.com", "nordaccount.com"], category_title: "VPN Service (NordVPN)" },
    AppSignature { name: "ExpressVPN", domain_id: "vpns_privacy", subcategory: "expressvpn", keywords: &["expressvpn.com"], category_title: "VPN Service (ExpressVPN)" },
    AppSignature { name: "Mullvad VPN", domain_id: "vpns_privacy", subcategory: "mullvad", keywords: &["mullvad.net"], category_title: "Anonymous VPN (Mullvad)" },
    AppSignature { name: "1Password", domain_id: "vpns_privacy", subcategory: "onepassword", keywords: &["1password.com"], category_title: "Password Manager (1Password)" },
    AppSignature { name: "Bitwarden", domain_id: "vpns_privacy", subcategory: "bitwarden", keywords: &["bitwarden.com"], category_title: "Password Manager (Bitwarden)" },

    // 🖥️ REMOTE ACCESS & PRODUCTIVITY
    AppSignature { name: "AnyDesk", domain_id: "remote_collab", subcategory: "anydesk", keywords: &["anydesk.com"], category_title: "Remote Desktop (AnyDesk)" },
    AppSignature { name: "TeamViewer", domain_id: "remote_collab", subcategory: "teamviewer", keywords: &["teamviewer.com"], category_title: "Remote Desktop (TeamViewer)" },
    AppSignature { name: "RustDesk", domain_id: "remote_collab", subcategory: "rustdesk", keywords: &["rustdesk.com"], category_title: "Remote Desktop (RustDesk)" },
    AppSignature { name: "Zoom", domain_id: "remote_collab", subcategory: "zoom", keywords: &["zoom.us"], category_title: "Video Meetings (Zoom)" },
    AppSignature { name: "Slack", domain_id: "remote_collab", subcategory: "slack", keywords: &["slack.com", "slackmail.com"], category_title: "Workspace (Slack)" },
    AppSignature { name: "Notion", domain_id: "remote_collab", subcategory: "notion", keywords: &["notion.so"], category_title: "Workspace (Notion)" },
    AppSignature { name: "Dropbox", domain_id: "remote_collab", subcategory: "dropbox", keywords: &["dropbox.com", "dropboxmail.com"], category_title: "Cloud Storage (Dropbox)" },
    AppSignature { name: "Google Drive", domain_id: "remote_collab", subcategory: "gdrive", keywords: &["drive.google.com", "docs.google.com"], category_title: "Cloud Storage (Google Drive)" },
    AppSignature { name: "Microsoft OneDrive", domain_id: "remote_collab", subcategory: "onedrive", keywords: &["onedrive.live.com", "sharepoint.com"], category_title: "Cloud Storage (OneDrive)" },
    AppSignature { name: "Mega.nz", domain_id: "remote_collab", subcategory: "mega", keywords: &["mega.nz", "mega.io"], category_title: "Encrypted Cloud (Mega)" },

    // 🎮 GAMING, ESPORTS & GAMBLING
    AppSignature { name: "Steam", domain_id: "gaming_gambling", subcategory: "steam", keywords: &["steampowered.com", "valvesoftware.com"], category_title: "Gaming Platform (Steam)" },
    AppSignature { name: "Epic Games", domain_id: "gaming_gambling", subcategory: "epic_games", keywords: &["epicgames.com"], category_title: "Gaming Store (Epic Games)" },
    AppSignature { name: "PlayStation", domain_id: "gaming_gambling", subcategory: "playstation", keywords: &["playstation.com", "sony.com"], category_title: "Console Network (PlayStation)" },
    AppSignature { name: "Xbox", domain_id: "gaming_gambling", subcategory: "xbox", keywords: &["xbox.com"], category_title: "Gaming Network (Xbox)" },
    AppSignature { name: "Roblox", domain_id: "gaming_gambling", subcategory: "roblox", keywords: &["roblox.com"], category_title: "Metaverse / Gaming (Roblox)" },
    AppSignature { name: "Stake.com", domain_id: "gaming_gambling", subcategory: "stake", keywords: &["stake.com"], category_title: "Crypto Gambling (Stake.com)" },
    AppSignature { name: "Bet365", domain_id: "gaming_gambling", subcategory: "bet365", keywords: &["bet365.com"], category_title: "Sportsbook (Bet365)" },
];

/// Helper to parse clean domain from email address or URL
fn extract_domain(email_or_url: &str) -> Option<String> {
    if let Some(pos) = email_or_url.find('@') {
        let domain_part = &email_or_url[pos + 1..];
        let clean = domain_part.trim().trim_matches(|c| c == '>' || c == '<' || c == ' ' || c == ';' || c == ',');
        if clean.contains('.') {
            return Some(clean.to_lowercase());
        }
    }
    None
}

/// Case Artifacts Summary by Taxonomy Domains (Hides 0-count domains by default)
#[tauri::command]
pub async fn case_artifacts_summary(
    state: State<'_, AppState>,
    input: Value,
) -> Result<Vec<TaxonomyDomainSummary>, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let show_all = input["show_all"].as_bool().unwrap_or(false);
    let all_artifacts = get_or_extract_artifacts(&state, &case_id, false).await?;

    let domain_defs = [
        // Digital Footprint & App Accounts (User Identified Priority)
        ("social_media", "Social Media & Communities", "🌐"),
        ("mobile_apps", "Mobile Apps & On-Demand", "📱"),
        ("crypto_platforms", "Crypto Exchanges & Web3", "🪙"),
        ("messaging_apps", "Encrypted & Instant Messengers", "💬"),
        ("dating_apps", "Dating & Romance Platforms", "❤️"),
        ("fintech_banking", "Fintech & Digital Banking", "🏦"),
        ("ecommerce", "E-Commerce & Marketplaces", "🛍️"),
        ("cloud_dev", "AI, Cloud & Developer Tools", "🤖"),
        ("vpn_privacy", "VPNs, Privacy & Anonymous Mail", "🛡️"),
        ("remote_access", "Remote Desktop & Collaboration", "🖥️"),
        ("gaming_gambling", "Gaming, Esports & Gambling", "🎮"),

        // Forensic Extractions
        ("credentials", "Credentials & Secrets", "🔑"),
        ("crypto", "Cryptocurrency & Seeds", "🪙"),
        ("financial", "Financial & Banking Numbers", "💳"),
        ("identity_docs", "PII & Identity Documents", "🪪"),
        ("locations", "Locations, Travel & Addresses", "📍"),
        ("contraband", "Threats & Contraband", "🛑"),
        ("malware_threats", "Malware & Cyber IOCs", "🦠"),
        ("secrets", "Corporate & Legal Privileged", "📄"),
        ("phishing", "Phishing & Social Engineering", "🎣"),
        ("network", "Suspicious Network & URL Hooks", "🌐"),
        ("attachments", "Carved & Suspicious Files", "📎"),
        ("deleted_recovered", "Deleted & Carved Messages", "🗑️"),
        ("authentication", "Failed Authentication & Spoofing", "🔐"),
        ("calendar", "Calendar & Meetings (.ics)", "📅"),
        ("client", "Email Clients & Devices", "💻"),
        ("containers", "Mailboxes & Containers", "🗂️"),
        ("case_artifacts", "Evidence Integrity Seals", "⚖️"),
    ];

    let mut result = Vec::new();
    let mut handled_domains = std::collections::HashSet::new();

    for (dom_id, dom_name, dom_icon) in &domain_defs {
        handled_domains.insert(dom_id.to_string());
        let domain_artifacts: Vec<&ForensicTaxonomyArtifact> = all_artifacts.iter().filter(|a| {
            a.domain_id == *dom_id || match *dom_id {
                "ecommerce" => a.domain_id == "ecommerce_shopping",
                "cloud_dev" => a.domain_id == "ai_cloud_dev",
                "vpn_privacy" => a.domain_id == "vpns_privacy",
                "remote_access" => a.domain_id == "remote_collab",
                _ => false,
            }
        }).collect();
        let total_count = domain_artifacts.len();

        if !show_all && total_count == 0 {
            continue;
        }

        let mut sub_map: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        for a in &domain_artifacts {
            *sub_map.entry(a.subcategory_id.clone()).or_insert(0) += 1;
        }

        let subcategories = sub_map.into_iter().filter(|(_, cnt)| *cnt > 0).map(|(k, v)| {
            let name = k.replace('_', " ").to_uppercase();
            TaxonomySubcategorySummary {
                subcategory_id: k,
                name,
                count: v,
            }
        }).collect();

        result.push(TaxonomyDomainSummary {
            domain_id: dom_id.to_string(),
            name: dom_name.to_string(),
            icon: dom_icon.to_string(),
            total_count,
            subcategories,
        });
    }

    let mut dynamic_map: std::collections::BTreeMap<String, Vec<&ForensicTaxonomyArtifact>> = std::collections::BTreeMap::new();
    for a in &all_artifacts {
        if !handled_domains.contains(&a.domain_id) {
            dynamic_map.entry(a.domain_id.clone()).or_default().push(a);
        }
    }
    for (dom_id, items) in dynamic_map {
        let total_count = items.len();
        if !show_all && total_count == 0 {
            continue;
        }
        let dom_name = dom_id.replace('_', " ").to_uppercase();
        result.push(TaxonomyDomainSummary {
            domain_id: dom_id.clone(),
            name: dom_name,
            icon: "📁".to_string(),
            total_count,
            subcategories: vec![],
        });
    }

    Ok(result)
}

/// Case Artifacts List filtered by domain, subcategory, search, or severity
#[tauri::command]
pub async fn case_artifacts_list(
    state: State<'_, AppState>,
    input: Value,
) -> Result<Vec<ForensicTaxonomyArtifact>, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let domain = input["domain"].as_str()
        .or_else(|| input["category"].as_str())
        .unwrap_or("all");
    let subcategory = input["subcategory"].as_str().unwrap_or("all");
    let search = input["search"].as_str().unwrap_or("").to_lowercase();
    let artifact_type = input["artifact_type"].as_str().unwrap_or("all");

    let all_artifacts = get_or_extract_artifacts(&state, &case_id, false).await?;

    let filtered = all_artifacts.into_iter().filter(|item| {
        if domain != "all" && item.domain_id != domain {
            return false;
        }
        if subcategory != "all" && item.subcategory_id != subcategory {
            return false;
        }
        if artifact_type != "all" && item.artifact_type != artifact_type {
            return false;
        }
        if !search.is_empty() {
            let val_m = item.primary_value.to_lowercase().contains(&search);
            let title_m = item.title.to_lowercase().contains(&search);
            let det_m = item.details.to_lowercase().contains(&search);
            let subj_m = item.email_subject.as_deref().unwrap_or("").to_lowercase().contains(&search);
            let from_m = item.email_from.to_lowercase().contains(&search);
            if !val_m && !title_m && !det_m && !subj_m && !from_m {
                return false;
            }
        }
        true
    }).collect();

    Ok(filtered)
}

#[tauri::command]
pub async fn rescan_case_artifacts(
    state: State<'_, AppState>,
    input: Value,
) -> Result<usize, String> {
    let case_id = input["case_id"].as_str()
        .or_else(|| input["caseId"].as_str())
        .or_else(|| input["input"]["case_id"].as_str())
        .or_else(|| input["input"]["caseId"].as_str())
        .or_else(|| input.as_str())
        .unwrap_or("")
        .to_string();

    let arts = get_or_extract_artifacts(&state, &case_id, true).await?;
    Ok(arts.len())
}

async fn get_or_extract_artifacts(
    state: &State<'_, AppState>,
    case_id: &str,
    force_rescan: bool,
) -> Result<Vec<ForensicTaxonomyArtifact>, String> {
    if !force_rescan {
        let db = state.db.lock().await;
        let mut stmt = db.conn.prepare("
            SELECT id, domain_id, subcategory_id, title, primary_value, secondary_value,
                   details, severity, artifact_type, confidence, email_id, email_subject, email_from, date_sent_utc
            FROM forensic_artifacts
            WHERE case_id = ?1
        ").map_err(|e| e.to_string())?;

        let cached = stmt.query_map([case_id], |row| {
            Ok(ForensicTaxonomyArtifact {
                id: row.get(0)?,
                domain_id: row.get(1)?,
                subcategory_id: row.get(2)?,
                title: row.get(3)?,
                primary_value: row.get(4)?,
                secondary_value: row.get(5)?,
                details: row.get(6)?,
                severity: row.get(7)?,
                artifact_type: row.get(8)?,
                confidence: row.get(9)?,
                email_id: row.get(10)?,
                email_subject: row.get(11)?,
                email_from: row.get(12)?,
                date_sent_utc: row.get(13)?,
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        return Ok(cached);
    }

    let extracted = extract_all_taxonomy_artifacts(state, case_id).await?;

    let mut db = state.db.lock().await;
    let tx = db.conn.transaction().map_err(|e| e.to_string())?;
    let _ = tx.execute("DELETE FROM forensic_artifacts WHERE case_id = ?1", [case_id]);

    {
        let mut stmt = tx.prepare("
            INSERT OR REPLACE INTO forensic_artifacts (id, case_id, domain_id, subcategory_id, title, primary_value, secondary_value, details, severity, artifact_type, confidence, email_id, email_subject, email_from, date_sent_utc)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
        ").map_err(|e| e.to_string())?;

        for art in &extracted {
            let _ = stmt.execute(rusqlite::params![
                art.id, case_id, art.domain_id, art.subcategory_id, art.title,
                art.primary_value, art.secondary_value, art.details, art.severity,
                art.artifact_type, art.confidence, art.email_id, art.email_subject,
                art.email_from, art.date_sent_utc
            ]);
        }
    }
    tx.commit().map_err(|e| e.to_string())?;

    Ok(extracted)
}

async fn extract_all_taxonomy_artifacts(
    state: &State<'_, AppState>,
    case_id: &str,
) -> Result<Vec<ForensicTaxonomyArtifact>, String> {
    let (emails, attachments, evidence_items) = {
        let db = state.db.lock().await;

        let mut stmt = db.conn.prepare("
            SELECT id, from_addr, from_display, to_addrs, cc_addrs, reply_to, subject, body_text, body_html, headers_raw, 
                   date_sent_utc, risk_score, is_deleted, deleted_recovered, folder_category, message_id, in_reply_to, msg_references
            FROM emails
            WHERE case_id = ?1
            ORDER BY date_sent_utc DESC
        ").map_err(|e| e.to_string())?;

        let emails = stmt.query_map([case_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, Option<i64>>(11)?.unwrap_or(0) as u8,
                row.get::<_, Option<i64>>(12)?.unwrap_or(0) != 0,
                row.get::<_, Option<i64>>(13)?.unwrap_or(0) != 0,
                row.get::<_, Option<String>>(14)?,
                row.get::<_, Option<String>>(15)?,
                row.get::<_, Option<String>>(16)?,
                row.get::<_, Option<String>>(17)?,
            ))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        // Fetch case attachments
        let mut att_stmt = db.conn.prepare("
            SELECT a.id, a.email_id, a.filename, a.sha256, a.mime_type, a.size_bytes, a.entropy, a.risk_flags,
                   e.subject, e.from_addr, e.date_sent_utc
            FROM attachments a
            JOIN emails e ON a.email_id = e.id
            WHERE e.case_id = ?1
        ").map_err(|e| e.to_string())?;

        let attachments = att_stmt.query_map([case_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?.unwrap_or_else(|| "attachment.bin".to_string()),
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "application/octet-stream".to_string()),
                row.get::<_, i64>(5)? as u64,
                row.get::<_, Option<f64>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?.unwrap_or_default(),
                row.get::<_, Option<String>>(10)?,
            ))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        let mut ev_stmt = db.conn.prepare("
            SELECT id, filename, format, sha256, size_bytes, source_description, acquired_at
            FROM evidence_items
            WHERE case_id = ?1
        ").map_err(|e| e.to_string())?;

        let evidence_items = ev_stmt.query_map([case_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "Evidence".to_string()),
                row.get::<_, Option<String>>(2)?.unwrap_or_else(|| "unknown".to_string()),
                row.get::<_, Option<String>>(3)?.unwrap_or_else(|| "unsealed".to_string()),
                row.get::<_, i64>(4)? as u64,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?.unwrap_or_default(),
            ))
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        (emails, attachments, evidence_items)
    };

    let mut artifacts: Vec<ForensicTaxonomyArtifact> = Vec::new();

    // 0. Case Evidence Containers & Hashes
    for (ev_id, filename, format, sha256, size_bytes, source_desc, acquired_at) in evidence_items {
        artifacts.push(ForensicTaxonomyArtifact {
            id: format!("ev-{}", ev_id),
            domain_id: "containers".to_string(),
            subcategory_id: format.to_lowercase(),
            title: format!("Evidence Container ({})", format.to_uppercase()),
            primary_value: filename.clone(),
            secondary_value: Some(format!("SHA-256: {}", sha256)),
            details: format!("Format: {} | Size: {} B | Acquired: {} | Source: {}", format, size_bytes, acquired_at, source_desc.unwrap_or_default()),
            severity: "info".to_string(),
            artifact_type: "native".to_string(),
            confidence: Some("high".to_string()),
            email_id: String::new(),
            email_subject: Some(format!("Evidence Container: {}", filename)),
            email_from: "Case Evidence Store".to_string(),
            date_sent_utc: Some(acquired_at.clone()),
        });

        artifacts.push(ForensicTaxonomyArtifact {
            id: format!("hash-{}", ev_id),
            domain_id: "case_artifacts".to_string(),
            subcategory_id: "sha256_hash".to_string(),
            title: "Cryptographic SHA-256 Integrity Seal".to_string(),
            primary_value: sha256.clone(),
            secondary_value: Some(filename),
            details: format!("Cryptographic SHA-256 evidence integrity seal established at acquisition on {}", acquired_at),
            severity: "info".to_string(),
            artifact_type: "native".to_string(),
            confidence: Some("high".to_string()),
            email_id: String::new(),
            email_subject: Some("Chain of Custody Hash Seal".to_string()),
            email_from: "Forensic Acquisition Engine".to_string(),
            date_sent_utc: Some(acquired_at),
        });
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // STRICT FORENSIC REGEX MATCHERS (Zero False-Alarm Target)
    // ─────────────────────────────────────────────────────────────────────────────
    
    // 1. Credentials & Secrets
    let re_cred_pair = regex::Regex::new(r"(?i)(?:username|user|login|email)[:=\s]+([a-zA-Z0-9._%+\-@]{3,50})\s*(?:password|pwd|pass)[:=\s]+([^\s,;]{4,50})").ok();
    let re_pass_standalone = regex::Regex::new(r"(?i)(?:password|passwd|passcode|secret\s*key)[:=\s]+([^\s,;]{6,60})").ok();
    let re_api_keys = regex::Regex::new(r"\b(AKIA[0-9A-Z]{16}|sk_live_[0-9a-zA-Z]{24,40}|ghp_[0-9a-zA-Z]{36}|AIza[0-9A-Za-z\-_]{35})\b").ok();
    let re_bearer = regex::Regex::new(r"Bearer\s+([A-Za-z0-9\-\._~\+\/]{25,}=*)").ok();
    let re_jwt = regex::Regex::new(r"(eyJ[A-Za-z0-9-_=]{15,}\.[A-Za-z0-9-_=]{15,}\.[A-Za-z0-9-_.+/=]{10,})").ok();
    let re_ssh_key = regex::Regex::new(r"-----BEGIN (?:RSA|DSA|EC|OPENSSH) PRIVATE KEY-----").ok();
    // BIP-39 Seed Phrase: MUST be preceded by seed phrase indicator AND contain at least 12 lowercase words
    let re_seed = regex::Regex::new(r"(?i)(?:seed\s*phrase|recovery\s*phrase|mnemonic\s*phrase|wallet\s*seed)[:=\-]?\s*([a-z]{3,10}(?:\s+[a-z]{3,10}){11,23})").ok();
    let re_privkey = regex::Regex::new(r"(?i)(?:private\s*key|privkey)[:=\s]+([0-9a-fA-F]{64})\b").ok();

    // 2. Financial & Banking (Strict Contextual Prefixes)
    let re_cc_spaced = regex::Regex::new(r"\b((?:4[0-9]{3}|5[1-5][0-9]{2}|6011|3[47][0-9]{2})[\s\-][0-9]{4}[\s\-][0-9]{4}[\s\-][0-9]{4})\b").ok();
    let re_cc_raw = regex::Regex::new(r"\b(4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|6011[0-9]{12})\b").ok();
    let re_routing = regex::Regex::new(r"(?i)(?:routing(?:\s*number|#)?|aba(?:\s*#|\s*no)?)\s*[:#=]?\s*(\b(?:0[1-9]|[123][0-9]|6[1-9]|7[0-2]|80)\d{7}\b)").ok();
    let re_iban = regex::Regex::new(r"(?i)(?:iban)\s*[:#=]?\s*([A-Z]{2}[0-9]{2}[A-Z0-9]{4}[0-9]{7}(?:[A-Z0-9]?){0,16})\b").ok();
    let re_swift = regex::Regex::new(r"(?i)(?:swift(?:\s*code|\s*bic)?|bic(?:\s*code)?|swift/bic)\s*[:#=]?\s*([A-Z]{6}[A-Z0-9]{2}(?:[A-Z0-9]{3})?)\b").ok();
    let re_account = regex::Regex::new(r"(?i)(?:bank\s*account(?:\s*number|\s*#)?|acct(?:\s*number|\s*#))\s*[:#=]?\s*([0-9]{8,17})\b").ok();
    let re_sort_code = regex::Regex::new(r"(?i)(?:sort\s*code|sort-code|sortcode)\s*[:#=]?\s*(\d{2}[-\s]?\d{2}[-\s]?\d{2})\b").ok();

    // 3. Cryptocurrency (Strict Address Check)
    let re_btc_legacy = regex::Regex::new(r"\b([13][a-km-zA-HJ-NP-Z1-9]{25,34})\b").ok();
    let re_btc_bech32 = regex::Regex::new(r"\b(bc1[a-zA-HJ-NP-Z0-9]{39,59})\b").ok();
    let re_eth = regex::Regex::new(r"\b(0x[a-fA-F0-9]{40})\b").ok();
    let re_tron = regex::Regex::new(r"\b(T[A-Za-z1-9]{33})\b").ok();
    let re_sol = regex::Regex::new(r"\b([1-9A-HJ-NP-Za-km-z]{32,44})\b").ok();
    let re_ltc = regex::Regex::new(r"\b([LM3][a-km-zA-HJ-NP-Z1-9]{25,34})\b").ok();
    let re_doge = regex::Regex::new(r"\b(D[A-Za-z1-9]{33})\b").ok();
    let re_xmr = regex::Regex::new(r"\b(4[0-9AB][1-9A-HJ-NP-Za-km-z]{93})\b").ok();
    let re_crypto_uri = regex::Regex::new(r"(?i)\b((?:bitcoin|ethereum|litecoin|doge|solana|monero):[a-zA-Z0-9?=_&%-]+)\b").ok();

    // 4. PII & Identity (Strict Context)
    let re_ssn = regex::Regex::new(r"\b(\d{3}-\d{2}-\d{4})\b").ok();
    let re_passport = regex::Regex::new(r"(?i)(?:passport(?:\s*#|\s*no|\s*number)?)\s*[:#=]?\s*([A-PR-WYa-pr-wy][0-9]{7,8})\b").ok();
    let re_driver_lic = regex::Regex::new(r"(?i)(?:driver'?s?\s*license|driving\s*licence)\s*(?:#|no|number)?[:=\s]*([A-Z0-9]{6,14})\b").ok();
    let re_ein = regex::Regex::new(r"(?i)(?:ein|federal\s*tax\s*id)\s*[:#=]?\s*(\d{2}-\d{7})\b").ok();

    // 5. Locations, Travel & Addresses
    let re_street_addr = regex::Regex::new(r"\b([0-9]{1,5}\s+[A-Z][a-zA-Z0-9\s.,]{2,30}\s+(?:Street|St\.|Avenue|Ave\.|Road|Rd\.|Boulevard|Blvd\.|Lane|Ln\.|Drive|Dr\.|Way|Court|Ct\.|Parkway|Pkwy\.|Suite\s+[0-9A-Z]+|Apt\.\s+[0-9A-Z]+))\b").ok();
    let re_hotel_conf = regex::Regex::new(r"(?i)(?:hotel|lodging|flight)\s*(?:confirmation|booking|reservation)\s*(?:#|no|number)?[:=\s]*([A-Z0-9]{6,12})\b").ok();
    let re_gps = regex::Regex::new(r"\b(-?[0-9]{1,2}\.[0-9]{4,8}\s*,\s*-?[0-9]{1,3}\.[0-9]{4,8})\b").ok();

    // 6. Threats & Contraband (High Precision Words)
    let re_weapons = regex::Regex::new(r"(?i)\b(glock|beretta|ar-15|ak-47|silencer|ghost\s*gun|auto\s*sear|firearm|pistol|carbine|smg|shotgun|rifle|revolver)\b").ok();
    let re_narcotics = regex::Regex::new(r"(?i)\b(cocaine|coke|heroin|fentanyl|methamphetamine|crystal\s*meth|mdma|ecstasy|oxycodone|percocet|xanax|alprazolam|ketamine)\b").ok();
    let re_explosives = regex::Regex::new(r"(?i)\b(bomb|explosive|detonator|c4|ied|suicide\s*vest|pipe\s*bomb|anthrax|ricin|poison)\b").ok();
    let re_terrorism = regex::Regex::new(r"(?i)\b(al-qaeda|al-qa'ida|boko\s*haram|hezbollah|hamas|jihadist|suicide\s*bomber|terrorist\s*cell|taliban)\b").ok();

    // 7. Malware & Cyber IOCs
    let re_cve = regex::Regex::new(r"(CVE-\d{4}-\d{4,7})").ok();
    let re_c2 = regex::Regex::new(r"(?i)\b(command\s*and\s*control|c2\s*server|c&c\s*server|reverse\s*shell|meterpreter)\b").ok();

    // 8. Corporate & Legal Privileged
    let re_confidential = regex::Regex::new(r"(?i)\b(strictly\s+confidential|attorney[- ]client\s+privilege|privileged\s*and\s*confidential|work\s*product\s*doctrine)\b").ok();
    let re_nda = regex::Regex::new(r"(?i)\b(non[- ]disclosure\s*agreement|\bnda\b|proprietary\s+and\s+confidential)\b").ok();

    // 9. Phishing & Social Engineering
    let re_phish_cred = regex::Regex::new(r"(?i)\b(verify\s*your\s*identity|confirm\s*your\s*password|update\s*your\s*account\s*credentials)\b").ok();
    let re_phish_finance = regex::Regex::new(r"(?i)\b(wire\s*transfer\s*urgently|urgent\s*wire\s*payment|purchase\s*gift\s*cards)\b").ok();

    // Process attachments artifacts
    for (att_id, email_id, filename, sha256, mime, size, entropy, risk_flags, subj, from_addr, date_sent) in attachments {
        let cat = classify_attachment_category(&filename, &mime, entropy, risk_flags.as_deref());
        let is_dangerous = cat == "dangerous";
        let ent_val = entropy.unwrap_or(0.0);
        let is_high_entropy = ent_val > 7.5;

        if is_dangerous || is_high_entropy || cat == "archives" {
            artifacts.push(ForensicTaxonomyArtifact {
                id: format!("att-{}", att_id),
                domain_id: "attachments".to_string(),
                subcategory_id: if is_high_entropy { "high_entropy".to_string() } else { cat.clone() },
                title: format!("Carved File: {}", filename),
                primary_value: filename.clone(),
                secondary_value: Some(format!("SHA-256: {}", sha256)),
                details: format!("MIME: {} | Size: {} B | Entropy: {:.2}{}", mime, size, ent_val, if is_high_entropy { " [HIGH ENTROPY / PACKED]" } else { "" }),
                severity: if is_dangerous || is_high_entropy { "critical".to_string() } else { "info".to_string() },
                artifact_type: "native".to_string(),
                confidence: Some("high".to_string()),
                email_id,
                email_subject: subj,
                email_from: from_addr,
                date_sent_utc: date_sent,
            });
        }
    }

    let mut seen: HashSet<String> = HashSet::new();

    for (eid, from_addr, _from_disp, to_addrs, _cc_addrs, _reply_to, subj_opt, body_opt, html_opt, headers_raw_opt, date_opt, _risk, is_del, is_soft_del, folder_opt, msg_id_opt, _in_reply_to_opt, _ref_opt) in emails {
        let from_lower = from_addr.to_lowercase();
        let subj = subj_opt.as_deref().unwrap_or("");
        let subj_lower = subj.to_lowercase();
        let body = body_opt.as_deref().unwrap_or("");
        let html = html_opt.as_deref().unwrap_or("");
        let headers_raw = headers_raw_opt.as_deref().unwrap_or("");
        let headers_lower = headers_raw.to_lowercase();
        let folder = folder_opt.as_deref().unwrap_or("inbox");
        let full_text = format!("{} {}", subj, body);
        let full_text_lower = full_text.to_lowercase();

        // ─────────────────────────────────────────────────────────────────────────
        // 0. APPS, SOCIAL MEDIA, MESSENGERS & CLOUD SERVICES SIGNATURE ENGINE
        // ─────────────────────────────────────────────────────────────────────────
        let mut app_matched = false;
        for sig in APP_SIGNATURES {
            let matched = sig.keywords.iter().any(|&kw| {
                from_lower.contains(kw) 
                || headers_lower.contains(kw) 
                || subj_lower.contains(kw)
                || full_text_lower.contains(kw)
            });

            if matched {
                app_matched = true;
                let key = format!("app:{}:{}", sig.domain_id, sig.name);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: sig.domain_id.to_string(),
                        subcategory_id: sig.subcategory.to_string(),
                        title: sig.category_title.to_string(),
                        primary_value: sig.name.to_string(),
                        secondary_value: Some(format!("User/Recipient: {}", to_addrs)),
                        details: format!("Target account footprint detected on '{}'. Subject: '{}'", sig.name, subj),
                        severity: "medium".to_string(),
                        artifact_type: "derived".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        // 0.1 Dynamic External Service Account Extractor (Catches all unknown services/apps)
        if !app_matched {
            if let Some(domain) = extract_domain(&from_addr) {
                // Ignore common corporate/personal email providers from being misclassified
                let generic_providers = ["gmail.com", "yahoo.com", "hotmail.com", "outlook.com", "aol.com", "icloud.com", "enron.com"];
                if !generic_providers.iter().any(|&gp| domain.ends_with(gp)) && domain.contains('.') {
                    let parts: Vec<&str> = domain.split('.').collect();
                    if parts.len() >= 2 {
                        let brand = parts[parts.len() - 2];
                        if brand.len() >= 3 && !brand.chars().all(|c| c.is_ascii_digit()) {
                            let brand_cap = format!("{}{}", brand[..1].to_uppercase(), &brand[1..]);
                            let key = format!("dynamic_app:{}", domain);
                            if seen.insert(key) {
                                artifacts.push(ForensicTaxonomyArtifact {
                                    id: generate_id(),
                                    domain_id: "mobile_apps".to_string(),
                                    subcategory_id: "external_services".to_string(),
                                    title: format!("Web & Cloud Service ({})", brand_cap),
                                    primary_value: domain.clone(),
                                    secondary_value: Some(format!("Target: {}", to_addrs)),
                                    details: format!("Digital account correspondence from '{}' to '{}'", domain, to_addrs),
                                    severity: "info".to_string(),
                                    artifact_type: "derived".to_string(),
                                    confidence: Some("medium".to_string()),
                                    email_id: eid.clone(),
                                    email_subject: subj_opt.clone(),
                                    email_from: from_addr.clone(),
                                    date_sent_utc: date_opt.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // 1. DELETED & CARVED MESSAGES (Only if actually deleted/recovered)
        let is_deleted = is_del || is_soft_del || folder == "trash" || folder == "deleted items" || folder == "soft_deleted";
        if is_deleted {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "deleted_recovered".to_string(),
                subcategory_id: "dumpster_carved".to_string(),
                title: "Deleted / Dumpster Carved Message".to_string(),
                primary_value: if subj.is_empty() { "(No Subject)".to_string() } else { subj.to_string() },
                secondary_value: Some(from_addr.clone()),
                details: format!("Recovered from folder: {} | MsgID: {}", folder, msg_id_opt.as_deref().unwrap_or("")),
                severity: "high".to_string(),
                artifact_type: "recovered".to_string(),
                confidence: Some("high".to_string()),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // 2. CALENDAR & MEETINGS (.ics)
        if headers_lower.contains("text/calendar") || full_text_lower.contains("begin:vcalendar") || subj_lower.contains("invitation:") {
            artifacts.push(ForensicTaxonomyArtifact {
                id: generate_id(),
                domain_id: "calendar".to_string(),
                subcategory_id: "meetings_ics".to_string(),
                title: "Calendar Meeting Invitation (.ics)".to_string(),
                primary_value: if subj.is_empty() { "Calendar Event".to_string() } else { subj.to_string() },
                secondary_value: Some(from_addr.clone()),
                details: "iCalendar / Outlook meeting request object".to_string(),
                severity: "info".to_string(),
                artifact_type: "native".to_string(),
                confidence: Some("high".to_string()),
                email_id: eid.clone(),
                email_subject: subj_opt.clone(),
                email_from: from_addr.clone(),
                date_sent_utc: date_opt.clone(),
            });
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 1. CREDENTIALS & SECRETS
        // ─────────────────────────────────────────────────────────────────────────
        if let Some(ref re) = re_cred_pair {
            for cap in re.captures_iter(&full_text) {
                let user_val = cap[1].trim().to_string();
                let pass_val = cap[2].trim().to_string();
                let key = format!("cred_pair:{}:{}", user_val, pass_val);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "credentials".to_string(),
                        subcategory_id: "credentials_pair".to_string(),
                        title: "Credential Pair (User + Pass)".to_string(),
                        primary_value: format!("User: {} | Pass: {}", user_val, pass_val),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Extracted Account Login: User='{}', Pass='{}'", user_val, pass_val),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_pass_standalone {
            for cap in re.captures_iter(&full_text) {
                let pass_val = cap[1].trim().to_string();
                if pass_val.len() >= 6 && !pass_val.contains(' ') {
                    let key = format!("pass:{}", pass_val);
                    if seen.insert(key) {
                        artifacts.push(ForensicTaxonomyArtifact {
                            id: generate_id(),
                            domain_id: "credentials".to_string(),
                            subcategory_id: "passwords".to_string(),
                            title: "Standalone Password".to_string(),
                            primary_value: format!("Password: {}", pass_val),
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Standalone password value: {}", pass_val),
                            severity: "critical".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_api_keys {
            for cap in re.captures_iter(&full_text) {
                let key_val = cap[1].to_string();
                let key = format!("api:{}", key_val);
                if seen.insert(key) {
                    let provider = if key_val.starts_with("AKIA") { "AWS Access Key" } else if key_val.starts_with("sk_live_") { "Stripe Live Key" } else if key_val.starts_with("ghp_") { "GitHub Token" } else { "Google Cloud API Key" };
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "credentials".to_string(),
                        subcategory_id: "api_keys".to_string(),
                        title: format!("API Key ({})", provider),
                        primary_value: key_val.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Extracted {} credential token", provider),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_bearer {
            for cap in re.captures_iter(&full_text) {
                let token = cap[1].to_string();
                let key = format!("bearer:{}", token);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "credentials".to_string(),
                        subcategory_id: "bearer_tokens".to_string(),
                        title: "Bearer Authorization Token".to_string(),
                        primary_value: format!("Bearer {}", &token[..token.len().min(40)]),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("OAuth/Bearer authorization token: {}", token),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_jwt {
            for cap in re.captures_iter(&full_text) {
                let jwt = cap[1].to_string();
                let key = format!("jwt:{}", jwt);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "credentials".to_string(),
                        subcategory_id: "jwt_tokens".to_string(),
                        title: "JSON Web Token (JWT)".to_string(),
                        primary_value: format!("JWT: {}", &jwt[..jwt.len().min(45)]),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("JSON Web Token (JWT) session credential: {}", jwt),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_ssh_key {
            if re.is_match(&full_text) {
                let key = "ssh_key_block".to_string();
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "credentials".to_string(),
                        subcategory_id: "ssh_keys".to_string(),
                        title: "SSH / OpenSSH Private Key Block".to_string(),
                        primary_value: "-----BEGIN PRIVATE KEY-----".to_string(),
                        secondary_value: Some(from_addr.clone()),
                        details: "Private cryptographic SSH key block exposed in message".to_string(),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        // BIP-39 Seed phrase: Must have 12+ words, no dictionary newsletters
        if let Some(ref re) = re_seed {
            for cap in re.captures_iter(&full_text) {
                let seed = cap[1].trim().to_string();
                let words: Vec<&str> = seed.split_whitespace().collect();
                if words.len() >= 12 && !seed.contains("word of the day") && !seed.contains("merriam") {
                    let key = format!("seed:{}", seed);
                    if seen.insert(key) {
                        artifacts.push(ForensicTaxonomyArtifact {
                            id: generate_id(),
                            domain_id: "crypto".to_string(),
                            subcategory_id: "seed_phrases".to_string(),
                            title: "BIP-39 Mnemonic Seed Phrase (12+ words)".to_string(),
                            primary_value: seed.clone(),
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Cryptocurrency recovery seed phrase ({} words): {}", words.len(), seed),
                            severity: "critical".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_privkey {
            for cap in re.captures_iter(&full_text) {
                let pkey = cap[1].trim().to_string();
                let key = format!("privkey:{}", pkey);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "private_keys".to_string(),
                        title: "Cryptocurrency Hex Private Key".to_string(),
                        primary_value: pkey.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Raw hex private key: {}", pkey),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 2. FINANCIAL & BANKING (Luhn + ABA Routing Validated)
        // ─────────────────────────────────────────────────────────────────────────
        if let Some(ref re) = re_cc_spaced {
            for cap in re.captures_iter(&full_text) {
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
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Luhn-Verified Credit Card Number ({})", card_type),
                            severity: "critical".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_cc_raw {
            for cap in re.captures_iter(&full_text) {
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
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Luhn-Verified Card Number: {}", cc_raw),
                            severity: "critical".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_routing {
            for cap in re.captures_iter(&full_text) {
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
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Verified US 9-digit ABA Bank Routing Number: {}", r_no),
                            severity: "critical".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_iban {
            for cap in re.captures_iter(&full_text) {
                let iban = cap[1].trim().to_string();
                let key = format!("iban:{}", iban);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "financial".to_string(),
                        subcategory_id: "iban".to_string(),
                        title: "IBAN Bank Account Number".to_string(),
                        primary_value: format!("IBAN: {}", iban),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("International Bank Account Number (IBAN): {}", iban),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_swift {
            for cap in re.captures_iter(&full_text) {
                let swift = cap[1].trim().to_string();
                let key = format!("swift:{}", swift);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "financial".to_string(),
                        subcategory_id: "swift_bic".to_string(),
                        title: "SWIFT / BIC Bank Code".to_string(),
                        primary_value: format!("SWIFT: {}", swift),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("International Bank SWIFT/BIC Identifier: {}", swift),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_account {
            for cap in re.captures_iter(&full_text) {
                let acc = cap[1].trim().to_string();
                let key = format!("acct:{}", acc);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "financial".to_string(),
                        subcategory_id: "account_numbers".to_string(),
                        title: "Bank Account Number".to_string(),
                        primary_value: format!("Account #: {}", acc),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Extracted financial account number: {}", acc),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_sort_code {
            for cap in re.captures_iter(&full_text) {
                let sort = cap[1].trim().to_string();
                let key = format!("sort:{}", sort);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "financial".to_string(),
                        subcategory_id: "sort_code".to_string(),
                        title: "UK / Ireland Bank Sort Code".to_string(),
                        primary_value: format!("Sort Code: {}", sort),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Bank clearing sort code: {}", sort),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        // Bank Statements & Financial E-Statements Detection
        if subj_lower.contains("statement") || subj_lower.contains("estatement") || subj_lower.contains("electronic statement") || full_text_lower.contains("electronic statement notification") || full_text_lower.contains("bank statement") {
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
                    secondary_value: Some(from_addr.clone()),
                    details: format!("Electronic bank statement notification from {} ({})", bank_name, from_addr),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 3. CRYPTOCURRENCY (Validated Base58 / Bech32 / EVM / SOL / LTC / DOGE / XMR)
        // ─────────────────────────────────────────────────────────────────────────
        if let Some(ref re) = re_btc_legacy {
            for cap in re.captures_iter(&full_text) {
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
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Verified Bitcoin Base58 address: {}", btc),
                            severity: "high".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_btc_bech32 {
            for cap in re.captures_iter(&full_text) {
                let btc = cap[1].to_string();
                let key = format!("btc_bech:{}", btc);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "bitcoin_bech32".to_string(),
                        title: "Bitcoin SegWit (Bech32) Address".to_string(),
                        primary_value: btc.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Bitcoin SegWit Native Bech32 Address: {}", btc),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_eth {
            for cap in re.captures_iter(&full_text) {
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
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Ethereum / EVM Address: {}", eth),
                            severity: "high".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_tron {
            for cap in re.captures_iter(&full_text) {
                let trx = cap[1].to_string();
                let key = format!("trx:{}", trx);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "tron".to_string(),
                        title: "TRON (TRX / USDT-TRC20) Address".to_string(),
                        primary_value: trx.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("TRON Network Address: {}", trx),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_sol {
            for cap in re.captures_iter(&full_text) {
                let sol = cap[1].to_string();
                if sol.len() >= 32 && sol.len() <= 44 && !sol.contains('@') {
                    let key = format!("sol:{}", sol);
                    if seen.insert(key) {
                        artifacts.push(ForensicTaxonomyArtifact {
                            id: generate_id(),
                            domain_id: "crypto".to_string(),
                            subcategory_id: "solana".to_string(),
                            title: "Solana (SOL) Wallet Address".to_string(),
                            primary_value: sol.clone(),
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Solana Blockchain Public Address: {}", sol),
                            severity: "high".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_ltc {
            for cap in re.captures_iter(&full_text) {
                let ltc = cap[1].to_string();
                let key = format!("ltc:{}", ltc);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "litecoin".to_string(),
                        title: "Litecoin (LTC) Address".to_string(),
                        primary_value: ltc.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Litecoin Network Address: {}", ltc),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_doge {
            for cap in re.captures_iter(&full_text) {
                let doge = cap[1].to_string();
                let key = format!("doge:{}", doge);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "dogecoin".to_string(),
                        title: "Dogecoin (DOGE) Address".to_string(),
                        primary_value: doge.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Dogecoin Network Address: {}", doge),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_xmr {
            for cap in re.captures_iter(&full_text) {
                let xmr = cap[1].to_string();
                let key = format!("xmr:{}", xmr);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "monero".to_string(),
                        title: "Monero (XMR) Privacy Address".to_string(),
                        primary_value: xmr.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Monero (XMR) Stealth Address: {}", xmr),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_crypto_uri {
            for cap in re.captures_iter(&full_text) {
                let uri = cap[1].to_string();
                let key = format!("crypto_uri:{}", uri);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "crypto".to_string(),
                        subcategory_id: "qr_wallet_uris".to_string(),
                        title: "Cryptocurrency Wallet Payment URI".to_string(),
                        primary_value: uri.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Payment URI schema: {}", uri),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 4. PERSONAL IDENTIFIABLE INFORMATION (PII) (Validated SSN / Passport / Real DL)
        // ─────────────────────────────────────────────────────────────────────────
        if let Some(ref re) = re_ssn {
            for cap in re.captures_iter(&full_text) {
                let ssn = cap[1].to_string();
                if validate_ssn(&ssn) {
                    let key = format!("ssn:{}", ssn);
                    if seen.insert(key) {
                        artifacts.push(ForensicTaxonomyArtifact {
                            id: generate_id(),
                            domain_id: "identity_docs".to_string(),
                            subcategory_id: "ssn".to_string(),
                            title: "US Social Security Number (SSN)".to_string(),
                            primary_value: ssn.clone(),
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Verified US Social Security Number: {}", ssn),
                            severity: "critical".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_passport {
            for cap in re.captures_iter(&full_text) {
                let pass = cap[1].to_string();
                let key = format!("passport:{}", pass);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "identity_docs".to_string(),
                        subcategory_id: "passport".to_string(),
                        title: "International Passport Number".to_string(),
                        primary_value: format!("Passport: {}", pass),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Passport document identifier: {}", pass),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_driver_lic {
            for cap in re.captures_iter(&full_text) {
                let dl = cap[1].trim().to_string();
                if dl.chars().any(|c| c.is_ascii_digit()) && dl.len() >= 6 {
                    let key = format!("dl:{}", dl);
                    if seen.insert(key) {
                        artifacts.push(ForensicTaxonomyArtifact {
                            id: generate_id(),
                            domain_id: "identity_docs".to_string(),
                            subcategory_id: "drivers_license".to_string(),
                            title: "Driver's License (DLN)".to_string(),
                            primary_value: format!("DL: {}", dl),
                            secondary_value: Some(from_addr.clone()),
                            details: format!("Driver's license identifier: {}", dl),
                            severity: "high".to_string(),
                            artifact_type: "native".to_string(),
                            confidence: Some("high".to_string()),
                            email_id: eid.clone(),
                            email_subject: subj_opt.clone(),
                            email_from: from_addr.clone(),
                            date_sent_utc: date_opt.clone(),
                        });
                    }
                }
            }
        }

        if let Some(ref re) = re_ein {
            for cap in re.captures_iter(&full_text) {
                let ein = cap[1].to_string();
                let key = format!("ein:{}", ein);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "identity_docs".to_string(),
                        subcategory_id: "ein".to_string(),
                        title: "Employer Identification Number (EIN)".to_string(),
                        primary_value: format!("EIN: {}", ein),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("US Federal Employer Identification Number: {}", ein),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 5. LOCATIONS, TRAVEL & ADDRESSES
        // ─────────────────────────────────────────────────────────────────────────
        if let Some(ref re) = re_street_addr {
            for cap in re.captures_iter(&body) {
                let addr = cap[1].trim().to_string();
                let key = format!("addr:{}", addr);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "locations".to_string(),
                        subcategory_id: "street_address".to_string(),
                        title: "Physical Street Address".to_string(),
                        primary_value: addr.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Street address: {}", addr),
                        severity: "medium".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        if let Some(ref re) = re_hotel_conf {
            for cap in re.captures_iter(&full_text) {
                let conf = cap[1].to_string();
                let key = format!("hotel_conf:{}", conf);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "locations".to_string(),
                        subcategory_id: "hotel_booking".to_string(),
                        title: "Travel / Lodging Confirmation".to_string(),
                        primary_value: format!("Booking #: {}", conf),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Travel lodging confirmation code: {}", conf),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_gps {
            for cap in re.captures_iter(&full_text) {
                let gps = cap[1].to_string();
                let key = format!("gps:{}", gps);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "locations".to_string(),
                        subcategory_id: "gps_coordinates".to_string(),
                        title: "GPS Geographic Coordinates".to_string(),
                        primary_value: gps.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Latitude / Longitude coordinates: {}", gps),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 6. THREATS & CONTRABAND (Precise Keywords)
        // ─────────────────────────────────────────────────────────────────────────
        if let Some(ref re) = re_weapons {
            for cap in re.captures_iter(&full_text) {
                let wpn = cap[1].to_string();
                let key = format!("wpn:{}", wpn.to_lowercase());
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "contraband".to_string(),
                        subcategory_id: "weapons".to_string(),
                        title: format!("Firearms & Weapons ({})", wpn.to_uppercase()),
                        primary_value: wpn.to_uppercase(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Firearm or weapons keyword: {}", wpn),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        if let Some(ref re) = re_narcotics {
            for cap in re.captures_iter(&full_text) {
                let drug = cap[1].to_string();
                let key = format!("drug:{}", drug.to_lowercase());
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "contraband".to_string(),
                        subcategory_id: "narcotics".to_string(),
                        title: format!("Controlled Substances ({})", drug.to_uppercase()),
                        primary_value: drug.to_uppercase(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Illicit drug mention: {}", drug),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        if let Some(ref re) = re_explosives {
            for cap in re.captures_iter(&full_text) {
                let exp = cap[1].to_string();
                let key = format!("exp:{}", exp.to_lowercase());
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "contraband".to_string(),
                        subcategory_id: "explosives".to_string(),
                        title: format!("Explosives & IED Threat ({})", exp.to_uppercase()),
                        primary_value: exp.to_uppercase(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Explosive material or detonator indicator: {}", exp),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        if let Some(ref re) = re_terrorism {
            for cap in re.captures_iter(&full_text) {
                let trr = cap[1].to_string();
                let key = format!("trr:{}", trr.to_lowercase());
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "contraband".to_string(),
                        subcategory_id: "terrorism".to_string(),
                        title: format!("Violent Extremism ({})", trr.to_uppercase()),
                        primary_value: trr.to_uppercase(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Terrorist organization or extremist keyword: {}", trr),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 7. MALWARE & CYBER IOCs
        // ─────────────────────────────────────────────────────────────────────────
        if let Some(ref re) = re_cve {
            for cap in re.captures_iter(&full_text) {
                let cve = cap[1].to_string();
                let key = format!("cve:{}", cve);
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "malware_threats".to_string(),
                        subcategory_id: "cve_vulnerability".to_string(),
                        title: format!("Common Vulnerability ({})", cve),
                        primary_value: cve.clone(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Vulnerability identifier: {}", cve),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                }
            }
        }

        if let Some(ref re) = re_c2 {
            for cap in re.captures_iter(&full_text) {
                let c2 = cap[1].to_string();
                let key = format!("c2:{}", c2.to_lowercase());
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "malware_threats".to_string(),
                        subcategory_id: "c2_indicators".to_string(),
                        title: "Command & Control (C2) Indicator".to_string(),
                        primary_value: c2.to_uppercase(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Command & Control terminology: {}", c2),
                        severity: "critical".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 8. CORPORATE & LEGAL PRIVILEGED
        // ─────────────────────────────────────────────────────────────────────────
        if let Some(ref re) = re_confidential {
            for cap in re.captures_iter(&full_text) {
                let conf = cap[1].to_string();
                let key = format!("confidential:{}", conf.to_lowercase());
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "secrets".to_string(),
                        subcategory_id: "privileged_confidential".to_string(),
                        title: format!("Legal Privilege / Confidential ({})", conf.to_uppercase()),
                        primary_value: conf.to_uppercase(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Confidentiality or legal privilege notice: {}", conf),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        if let Some(ref re) = re_nda {
            for cap in re.captures_iter(&full_text) {
                let nda_val = cap[1].to_string();
                let key = format!("nda:{}", nda_val.to_lowercase());
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "secrets".to_string(),
                        subcategory_id: "nda_agreements".to_string(),
                        title: "Non-Disclosure Agreement (NDA)".to_string(),
                        primary_value: nda_val.to_uppercase(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("NDA or distribution restriction clause: {}", nda_val),
                        severity: "high".to_string(),
                        artifact_type: "native".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 9. PHISHING & SOCIAL ENGINEERING
        // ─────────────────────────────────────────────────────────────────────────
        if let Some(ref re) = re_phish_cred {
            for cap in re.captures_iter(&full_text) {
                let cr = cap[1].to_string();
                let key = format!("phish_cred:{}", cr.to_lowercase());
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "phishing".to_string(),
                        subcategory_id: "credential_requests".to_string(),
                        title: "Credential Harvesting Lure".to_string(),
                        primary_value: cr.to_uppercase(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Request for login credentials / password update: {}", cr),
                        severity: "critical".to_string(),
                        artifact_type: "derived".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        if let Some(ref re) = re_phish_finance {
            for cap in re.captures_iter(&full_text) {
                let fin = cap[1].to_string();
                let key = format!("phish_fin:{}", fin.to_lowercase());
                if seen.insert(key) {
                    artifacts.push(ForensicTaxonomyArtifact {
                        id: generate_id(),
                        domain_id: "phishing".to_string(),
                        subcategory_id: "financial_demands".to_string(),
                        title: "BEC / Financial Payment Demand".to_string(),
                        primary_value: fin.to_uppercase(),
                        secondary_value: Some(from_addr.clone()),
                        details: format!("Fraudulent wire transfer or gift card demand: {}", fin),
                        severity: "critical".to_string(),
                        artifact_type: "derived".to_string(),
                        confidence: Some("high".to_string()),
                        email_id: eid.clone(),
                        email_subject: subj_opt.clone(),
                        email_from: from_addr.clone(),
                        date_sent_utc: date_opt.clone(),
                    });
                    break;
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // 10. AUTHENTICATION FAILURES (Only report actual FAILURES, not every email)
        // ─────────────────────────────────────────────────────────────────────────
        if headers_lower.contains("spf=fail") || headers_lower.contains("spf=softfail") {
            let key = "spf_fail".to_string();
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "authentication".to_string(),
                    subcategory_id: "spf_fail".to_string(),
                    title: "SPF Authentication Failure".to_string(),
                    primary_value: "SPF: FAIL".to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: "Sender failed SPF domain authorization check".to_string(),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if headers_lower.contains("dkim=fail") {
            let key = "dkim_fail".to_string();
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "authentication".to_string(),
                    subcategory_id: "dkim_fail".to_string(),
                    title: "DKIM Cryptographic Signature Failure".to_string(),
                    primary_value: "DKIM: FAIL".to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: "Cryptographic signature validation failed on transport header".to_string(),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        if headers_lower.contains("dmarc=fail") {
            let key = "dmarc_fail".to_string();
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "authentication".to_string(),
                    subcategory_id: "dmarc_fail".to_string(),
                    title: "DMARC Alignment Policy Failure".to_string(),
                    primary_value: "DMARC: FAIL".to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: "Message failed DMARC domain alignment policy".to_string(),
                    severity: "critical".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }

        // 11. Hidden 1x1 Web Tracking Beacons
        if html.contains("width=\"1\" height=\"1\"") || html.contains("width='1' height='1'") {
            let key = "tracking_pixel".to_string();
            if seen.insert(key) {
                artifacts.push(ForensicTaxonomyArtifact {
                    id: generate_id(),
                    domain_id: "network".to_string(),
                    subcategory_id: "tracking_pixels".to_string(),
                    title: "Hidden 1x1 Web Tracking Beacon".to_string(),
                    primary_value: "1x1 Web Beacon".to_string(),
                    secondary_value: Some(from_addr.clone()),
                    details: "Hidden 1x1 tracking beacon embedded in HTML".to_string(),
                    severity: "medium".to_string(),
                    artifact_type: "native".to_string(),
                    confidence: Some("high".to_string()),
                    email_id: eid.clone(),
                    email_subject: subj_opt.clone(),
                    email_from: from_addr.clone(),
                    date_sent_utc: date_opt.clone(),
                });
            }
        }
    }

    Ok(artifacts)
}
