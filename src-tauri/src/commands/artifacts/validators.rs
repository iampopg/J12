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

/// Generic Base58 Decoded Byte Array Generator
pub fn decode_base58(addr: &str) -> Option<Vec<u8>> {
    let alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut decoded = vec![0u8; 64];
    let mut decoded_len = 0;

    for c in addr.chars() {
        let mut carry = alphabet.find(c)? as u32;
        for i in 0..decoded_len {
            carry += (decoded[i] as u32) * 58;
            decoded[i] = (carry & 0xFF) as u8;
            carry >>= 8;
        }
        while carry > 0 {
            if decoded_len >= 64 { return None; }
            decoded[decoded_len] = (carry & 0xFF) as u8;
            decoded_len += 1;
            carry >>= 8;
        }
    }
    for c in addr.chars() {
        if c == '1' {
            if decoded_len >= 64 { return None; }
            decoded[decoded_len] = 0;
            decoded_len += 1;
        } else {
            break;
        }
    }
    decoded.truncate(decoded_len);
    decoded.reverse();
    Some(decoded)
}

/// Solana (SOL) 32-Byte Ed25519 Public Key Base58 Validator
pub fn validate_solana_base58(addr: &str) -> bool {
    if addr.len() < 32 || addr.len() > 44 { return false; }
    if let Some(bytes) = decode_base58(addr) {
        bytes.len() == 32
    } else {
        false
    }
}

/// TRON (TRX / USDT-TRC20) Base58Check Address Validator
pub fn validate_tron_base58(addr: &str) -> bool {
    if addr.len() != 34 || !addr.starts_with('T') { return false; }
    if let Some(bytes) = decode_base58(addr) {
        if bytes.len() != 25 { return false; }
        use sha2::{Sha256, Digest};
        let hash1 = Sha256::digest(&bytes[0..21]);
        let hash2 = Sha256::digest(&hash1);
        &hash2[0..4] == &bytes[21..25]
    } else {
        false
    }
}

/// Phone Number Sanitizer & Quality Check
#[allow(dead_code)]
pub fn validate_phone(p: &str) -> bool {
    let digits: Vec<char> = p.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 10 || digits.len() > 15 { return false; }
    if digits.iter().all(|&c| c == digits[0]) { return false; }
    if (p.starts_with("19") || p.starts_with("20")) && p.contains('-') && digits.len() <= 8 {
        return false;
    }
    true
}

/// ISO 7064 Mod 97-10 IBAN Checksum Validator
pub fn validate_iban(iban: &str) -> bool {
    let clean: String = iban.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_uppercase()).collect();
    if clean.len() < 15 || clean.len() > 34 {
        return false;
    }
    let chars: Vec<char> = clean.chars().collect();
    if !chars[0].is_ascii_alphabetic() || !chars[1].is_ascii_alphabetic() || !chars[2].is_ascii_digit() || !chars[3].is_ascii_digit() {
        return false;
    }

    let rearranged = format!("{}{}", &clean[4..], &clean[0..4]);
    let mut remainder = 0u64;

    for c in rearranged.chars() {
        if c.is_ascii_digit() {
            let digit = c.to_digit(10).unwrap() as u64;
            remainder = (remainder * 10 + digit) % 97;
        } else if c.is_ascii_alphabetic() {
            let val = c as u64 - 'A' as u64 + 10;
            remainder = (remainder * 100 + val) % 97;
        } else {
            return false;
        }
    }

    remainder == 1
}

/// ISO 9362 SWIFT / BIC Code Structure Validator
pub fn validate_swift_bic(swift: &str) -> bool {
    let clean: String = swift.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_uppercase()).collect();
    if clean.len() != 8 && clean.len() != 11 {
        return false;
    }
    let chars: Vec<char> = clean.chars().collect();
    if !chars[0..4].iter().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }
    if !chars[4..6].iter().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }
    if !chars[6..8].iter().all(|c| c.is_ascii_alphanumeric()) {
        return false;
    }
    true
}

