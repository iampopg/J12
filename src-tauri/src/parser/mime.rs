use super::types::RawAttachment;

/// Parse multipart MIME body into text, html, and attachments
pub fn parse_multipart(body: &str, boundary: &str) -> (Option<String>, Option<String>, Vec<RawAttachment>) {
    let mut text_parts = Vec::new();
    let mut html_parts = Vec::new();
    let mut attachments = Vec::new();
    
    let delimiter = format!("--{}", boundary);
    
    let parts: Vec<&str> = body.split(&delimiter).collect();
    
    for part in &parts {
        let part = part.trim_start_matches("\r\n").trim_start_matches("\n");
        if part.is_empty() || part == "--" {
            continue;
        }
        
        let (header_section, body_content) = if let Some(idx) = part.find("\r\n\r\n") {
            (&part[..idx], &part[idx+4..])
        } else if let Some(idx) = part.find("\n\n") {
            (&part[..idx], &part[idx+2..])
        } else {
            continue;
        };
        
        let mut part_content_type = String::new();
        let mut part_encoding = String::new();
        let mut part_filename = None;
        let mut part_name = None;
        let mut is_inline = false;
        
        for line in header_section.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value.trim();
                match key.as_str() {
                    "content-type" => {
                        part_content_type = value.split(';').next().unwrap_or(value).trim().to_lowercase();
                        if let Some(idx) = value.find("name=") {
                            let name_rest = &value[idx + 5..];
                            part_name = Some(name_rest.trim_matches('"').trim().to_string());
                        }
                    }
                    "content-transfer-encoding" => part_encoding = value.to_lowercase(),
                    "content-disposition" => {
                        let disp_lower = value.to_lowercase();
                        if disp_lower.contains("inline") {
                            is_inline = true;
                        }
                        if let Some(idx) = value.find("filename=") {
                            let fname_rest = &value[idx + 9..];
                            part_filename = Some(fname_rest.trim_matches('"').trim_matches('\'').trim().to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
        
        let decoded = if part_encoding.contains("base64") {
            base64_decode(body_content.trim())
        } else if part_encoding.contains("quoted-printable") {
            qp_decode(body_content)
        } else {
            body_content.as_bytes().to_vec()
        };
        
        if part_content_type.starts_with("multipart/") {
            if let Some(inner_boundary) = extract_boundary(&part_content_type) {
                let (t, h, a) = parse_multipart(body_content, &inner_boundary);
                if let Some(t) = t { text_parts.push(t); }
                if let Some(h) = h { html_parts.push(h); }
                attachments.extend(a);
            }
        } else if part_content_type.starts_with("text/plain") {
            text_parts.push(String::from_utf8_lossy(&decoded).to_string());
        } else if part_content_type.starts_with("text/html") {
            html_parts.push(String::from_utf8_lossy(&decoded).to_string());
        } else if part_content_type.starts_with("image/") || part_content_type.starts_with("application/") {
            let filename = part_filename.or(part_name).or_else(|| {
                let ext = match part_content_type.as_str() {
                    "image/jpeg" | "image/jpg" => "jpg",
                    "image/png" => "png",
                    "image/gif" => "gif",
                    "application/pdf" => "pdf",
                    "application/zip" => "zip",
                    "application/msword" => "doc",
                    _ => "bin",
                };
                Some(format!("attachment_{}.{}", attachments.len() + 1, ext))
            });
            
            attachments.push(RawAttachment {
                filename,
                content_type: part_content_type,
                data: decoded,
                is_inline,
            });
        } else if !decoded.is_empty() {
            let filename = part_filename.or(part_name);
            attachments.push(RawAttachment {
                filename,
                content_type: part_content_type,
                data: decoded,
                is_inline,
            });
        }
    }
    
    (
        if text_parts.is_empty() { None } else { Some(text_parts.join("\n")) },
        if html_parts.is_empty() { None } else { Some(html_parts.join("\n")) },
        attachments,
    )
}

pub fn base64_decode(input: &str) -> Vec<u8> {
    let cleaned: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &cleaned) {
        Ok(data) => data,
        Err(_) => cleaned.as_bytes().to_vec(),
    }
}

#[inline]
fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

pub fn qp_decode(input: &str) -> Vec<u8> {
    let mut result = Vec::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    let len = bytes.len();

    while i < len {
        if bytes[i] == b'=' {
            if i + 2 < len && bytes[i + 1] == b'\r' && bytes[i + 2] == b'\n' {
                // Soft line break =\r\n
                i += 3;
                continue;
            } else if i + 1 < len && (bytes[i + 1] == b'\n' || bytes[i + 1] == b'\r') {
                // Soft line break =\n or =\r
                i += 2;
                continue;
            } else if i + 2 < len {
                let h1 = bytes[i + 1];
                let h2 = bytes[i + 2];
                if let (Some(d1), Some(d2)) = (hex_val(h1), hex_val(h2)) {
                    result.push((d1 << 4) | d2);
                    i += 3;
                    continue;
                }
            }
            // If not a valid hex or soft line break, keep the '='
            result.push(b'=');
            i += 1;
        } else {
            result.push(bytes[i]);
            i += 1;
        }
    }

    result
}

pub fn qp_decode_str(input: &str) -> String {
    let bytes = qp_decode(input);
    String::from_utf8_lossy(&bytes).to_string()
}

pub fn decode_mime_word(s: &str) -> String {
    let mut result = String::new();
    let mut remaining = s;
    
    while let Some(start) = remaining.find("=?") {
        result.push_str(&remaining[..start]);
        
        if let Some(end) = remaining[start..].find("?=") {
            let encoded = &remaining[start..start + end + 2];
            let decoded = decode_single_mime_word(encoded);
            result.push_str(&decoded);
            remaining = &remaining[start + end + 2..];
        } else {
            break;
        }
    }
    result.push_str(remaining);
    result.trim().to_string()
}

pub fn decode_single_mime_word(s: &str) -> String {
    let s = s.trim_start_matches("=?").trim_end_matches("?=");
    let parts: Vec<&str> = s.split('?').collect();
    if parts.len() != 3 { return s.to_string(); }
    
    let _charset = parts[0];
    let encoding = parts[1].to_uppercase();
    let text = parts[2];
    
    if encoding == "B" {
        let cleaned: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &cleaned) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
            Err(_) => text.to_string(),
        }
    } else if encoding == "Q" {
        let bytes = qp_decode(text);
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        text.to_string()
    }
}

pub fn extract_boundary(content_type: &str) -> Option<String> {
    if let Some(idx) = content_type.find("boundary=") {
        let rest = &content_type[idx + 9..];
        let boundary = rest.trim_matches('"').trim_matches('\'').trim();
        if !boundary.is_empty() {
            return Some(boundary.to_string());
        }
    }
    None
}
