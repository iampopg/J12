use super::types::AttachmentAnalysis;

/// Analyze an attachment for forensic indicators
pub fn analyze_attachment(
    filename: Option<&str>,
    declared_mime: &str,
    data: &[u8],
) -> AttachmentAnalysis {
    let detected_type = detect_file_type(data);
    let extension_match = check_extension_match(filename, &detected_type);
    let entropy = calculate_entropy(data);
    
    let mut risk_flags = Vec::new();
    let mut risk_score: u8 = 0;
    
    if !extension_match && filename.is_some() {
        risk_flags.push("extension_mismatch".to_string());
        risk_score += 20;
    }
    
    if let Some(name) = filename {
        let lower = name.to_lowercase();
        let dangerous_exts = [".exe", ".scr", ".pif", ".cmd", ".bat", ".com", ".vbs", ".js", ".wsf", ".ps1", ".msi"];
        for ext in &dangerous_exts {
            if lower.ends_with(ext) {
                risk_flags.push(format!("dangerous_extension: {}", ext));
                risk_score += 30;
                break;
            }
        }
        
        let parts: Vec<&str> = lower.split('.').collect();
        if parts.len() > 2 {
            let second_ext = format!(".{}", parts[parts.len() - 2]);
            if dangerous_exts.contains(&second_ext.as_str()) || second_ext == ".pdf" || second_ext == ".doc" || second_ext == ".xls" {
                risk_flags.push("double_extension".to_string());
                risk_score += 40;
            }
        }
        
        if !lower.contains('.') {
            risk_flags.push("no_extension".to_string());
            risk_score += 10;
        }
    }
    
    if entropy > 7.5 {
        risk_flags.push("high_entropy: possibly encrypted".to_string());
        risk_score += 25;
    } else if entropy > 7.0 {
        risk_flags.push("elevated_entropy".to_string());
        risk_score += 10;
    }
    
    if is_office_document_with_macros(data) {
        risk_flags.push("office_macros_detected".to_string());
        risk_score += 35;
    }
    
    if detected_type == "application/x-dosexec" && 
       (declared_mime.contains("pdf") || declared_mime.contains("office") || declared_mime.contains("document")) {
        risk_flags.push("executable_disguised_as_document".to_string());
        risk_score += 50;
    }
    
    AttachmentAnalysis {
        filename: filename.map(|s| s.to_string()),
        declared_mime: declared_mime.to_string(),
        detected_type,
        extension_match,
        entropy,
        risk_flags,
        risk_score: risk_score.min(100),
    }
}

/// Analyze an attachment from database record metadata (entropy, filename, declared mime, risk_flags)
pub fn analyze_attachment_metadata(
    filename: Option<&str>,
    declared_mime: Option<&str>,
    _size_bytes: u64,
    entropy: Option<f64>,
    existing_flags: Option<&str>,
) -> AttachmentAnalysis {
    let mut risk_flags = Vec::new();
    let mut risk_score: u8 = 0;

    let mime = declared_mime.unwrap_or("application/octet-stream");
    let ent = entropy.unwrap_or(0.0);

    if let Some(name) = filename {
        let lower = name.to_lowercase();
        let dangerous_exts = [
            ".exe", ".scr", ".pif", ".cmd", ".bat", ".com", ".vbs", ".js", ".wsf", 
            ".ps1", ".msi", ".iso", ".hta", ".cpl", ".jar", ".reg"
        ];
        for ext in &dangerous_exts {
            if lower.ends_with(ext) {
                risk_flags.push(format!("dangerous_executable_extension: {}", ext));
                risk_score += 45;
                break;
            }
        }

        let macro_exts = [".docm", ".xlsm", ".pptm", ".dotm", ".xltm"];
        for ext in &macro_exts {
            if lower.ends_with(ext) {
                risk_flags.push("macro_enabled_office_document".to_string());
                risk_score += 35;
                break;
            }
        }

        let parts: Vec<&str> = lower.split('.').collect();
        if parts.len() > 2 {
            let second_ext = format!(".{}", parts[parts.len() - 2]);
            if dangerous_exts.contains(&second_ext.as_str()) || second_ext == ".pdf" || second_ext == ".doc" || second_ext == ".xls" {
                risk_flags.push(format!("double_extension_lure ({})", second_ext));
                risk_score += 45;
            }
        }
    }

    if ent > 7.5 {
        risk_flags.push(format!("high_entropy ({:.2}): probable packed/encrypted payload", ent));
        risk_score += 35;
    } else if ent > 7.1 {
        risk_flags.push(format!("elevated_entropy ({:.2})", ent));
        risk_score += 15;
    }

    if let Some(flags) = existing_flags {
        if !flags.trim().is_empty() && flags != "[]" {
            risk_flags.push(format!("risk_indicator: {}", flags));
            risk_score += 20;
        }
    }

    AttachmentAnalysis {
        filename: filename.map(|s| s.to_string()),
        declared_mime: mime.to_string(),
        detected_type: mime.to_string(),
        extension_match: true,
        entropy: ent,
        risk_flags,
        risk_score: risk_score.min(100),
    }
}

pub fn detect_file_type(data: &[u8]) -> String {
    if data.len() < 4 {
        return "application/octet-stream".to_string();
    }
    
    match &data[0..4] {
        [0x25, 0x50, 0x44, 0x46] => "application/pdf".to_string(),           // %PDF
        [0x50, 0x4b, 0x03, 0x04] => {
            if data.windows(4).any(|w| w == b"[Con") {
                return "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string();
            }
            "application/zip".to_string()
        },
        [0xd0, 0xcf, 0x11, 0xe0] => {
            "application/vnd.ms-office".to_string()
        },
        [0x7f, 0x45, 0x4c, 0x46] => "application/x-elf".to_string(),
        [0x4d, 0x5a, 0x90, 0x00] | [0x4d, 0x5a, 0x00, 0x00] => "application/x-dosexec".to_string(),
        [0x52, 0x61, 0x72, 0x21] => "application/x-rar-compressed".to_string(),
        [0x1f, 0x8b, 0x08, _] => "application/gzip".to_string(),
        [0x42, 0x5a, 0x68, _] => "application/x-bzip2".to_string(),
        [0xFD, 0x37, 0x7A, 0x58] => "application/x-xz".to_string(),
        [0x89, 0x50, 0x4E, 0x47] => "image/png".to_string(),
        [0xFF, 0xD8, 0xFF, _] => "image/jpeg".to_string(),
        [0x47, 0x49, 0x46, 0x38] => "image/gif".to_string(),
        [0x42, 0x4d, _, _] => "image/bmp".to_string(),
        [0x49, 0x20, 0x49, 0x00] | [0x49, 0x49, 0x2a, 0x00] => "image/tiff".to_string(),
        _ => {
            if data.iter().all(|&b| b == b'\n' || b == b'\r' || b == b'\t' || (b >= 0x20 && b < 0x7f)) {
                "text/plain".to_string()
            } else {
                "application/octet-stream".to_string()
            }
        }
    }
}

pub fn check_extension_match(filename: Option<&str>, detected_type: &str) -> bool {
    let ext = match filename {
        Some(name) => {
            name.rfind('.').map(|i| &name[i..]).unwrap_or("")
        }
        None => return true,
    };
    
    let ext_lower = ext.to_lowercase();
    
    match detected_type {
        "application/pdf" => ext_lower == ".pdf",
        "application/zip" => ext_lower == ".zip" || ext_lower == ".jar" || ext_lower == ".war",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => 
            ext_lower == ".docx" || ext_lower == ".xlsx" || ext_lower == ".pptx",
        "application/vnd.ms-office" => ext_lower == ".doc" || ext_lower == ".xls" || ext_lower == ".ppt" || ext_lower == ".msg",
        "application/x-dosexec" => ext_lower == ".exe" || ext_lower == ".dll" || ext_lower == ".sys",
        "application/x-rar-compressed" => ext_lower == ".rar",
        "application/gzip" => ext_lower == ".gz" || ext_lower == ".tgz" || ext_lower.ends_with(".tar.gz"),
        "image/png" => ext_lower == ".png",
        "image/jpeg" => ext_lower == ".jpg" || ext_lower == ".jpeg",
        "image/gif" => ext_lower == ".gif",
        "text/plain" => ext_lower == ".txt" || ext_lower == ".log" || ext_lower == ".csv" || ext_lower == ".text",
        _ => true,
    }
}

pub fn calculate_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    
    let mut freq = [0u64; 256];
    for &byte in data {
        freq[byte as usize] += 1;
    }
    
    let len = data.len() as f64;
    let mut entropy = 0.0;
    
    for &count in &freq {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }
    
    entropy
}

pub fn is_office_document_with_macros(data: &[u8]) -> bool {
    if data.len() > 8 && data[0..4] == [0xd0, 0xcf, 0x11, 0xe0] {
        data.windows(20).any(|window| {
            window.windows(11).any(|w| w.eq_ignore_ascii_case(b"VBAProject"))
        })
    } else {
        false
    }
}
