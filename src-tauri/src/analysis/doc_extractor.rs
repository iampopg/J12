use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn extract_document_text(path: &Path, filename: &str, mime_type: Option<&str>) -> Result<String, String> {
    let lower_name = filename.to_lowercase();
    let mime = mime_type.unwrap_or("").to_lowercase();

    if lower_name.ends_with(".txt") || lower_name.ends_with(".log") || lower_name.ends_with(".csv") || lower_name.ends_with(".tsv") || lower_name.ends_with(".json") || lower_name.ends_with(".xml") || mime.contains("text/plain") || mime.contains("text/csv") {
        extract_plain_text(path)
    } else if lower_name.ends_with(".html") || lower_name.ends_with(".htm") || mime.contains("text/html") {
        extract_html_text(path)
    } else if lower_name.ends_with(".rtf") || mime.contains("application/rtf") {
        extract_rtf_text(path)
    } else if lower_name.ends_with(".docx") || lower_name.ends_with(".docm") || mime.contains("wordprocessingml") {
        extract_docx_text(path)
    } else if lower_name.ends_with(".xlsx") || lower_name.ends_with(".xlsm") || mime.contains("spreadsheetml") {
        extract_xlsx_text(path)
    } else if lower_name.ends_with(".pptx") || lower_name.ends_with(".pptm") || mime.contains("presentationml") {
        extract_pptx_text(path)
    } else if lower_name.ends_with(".pdf") || mime.contains("application/pdf") {
        extract_pdf_text(path)
    } else {
        extract_plain_text(path)
    }
}

fn extract_plain_text(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| format!("Cannot open file: {}", e))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(|e| format!("Cannot read file: {}", e))?;

    if let Ok(s) = String::from_utf8(buf.clone()) {
        Ok(s)
    } else {
        // Fallback ASCII lossy conversion
        Ok(String::from_utf8_lossy(&buf).to_string())
    }
}

fn extract_html_text(path: &Path) -> Result<String, String> {
    let raw = extract_plain_text(path)?;
    let mut result = String::new();
    let mut in_tag = false;

    for c in raw.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
            result.push(' ');
        } else if !in_tag {
            result.push(c);
        }
    }

    Ok(result.split_whitespace().collect::<Vec<&str>>().join(" "))
}

fn extract_rtf_text(path: &Path) -> Result<String, String> {
    let raw = extract_plain_text(path)?;
    let mut result = String::new();
    let mut in_control = false;

    for c in raw.chars() {
        if c == '\\' {
            in_control = true;
        } else if in_control && (c == ' ' || c == '\n' || c == '\r' || c == ';') {
            in_control = false;
            if c == ' ' { result.push(' '); }
        } else if !in_control && c != '{' && c != '}' {
            result.push(c);
        }
    }

    Ok(result.trim().to_string())
}

fn extract_docx_text(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| format!("Cannot open DOCX: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid DOCX zip archive: {}", e))?;

    let mut doc_file = archive.by_name("word/document.xml").map_err(|e| format!("No word/document.xml in DOCX: {}", e))?;
    let mut xml_content = String::new();
    doc_file.read_to_string(&mut xml_content).map_err(|e| format!("Error reading document.xml: {}", e))?;

    Ok(parse_xml_text_tags(&xml_content, "w:t"))
}

fn extract_xlsx_text(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| format!("Cannot open XLSX: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid XLSX zip archive: {}", e))?;

    let mut text_parts = Vec::new();

    // Extract shared strings
    if let Ok(mut ss_file) = archive.by_name("xl/sharedStrings.xml") {
        let mut xml_content = String::new();
        if ss_file.read_to_string(&mut xml_content).is_ok() {
            let shared = parse_xml_text_tags(&xml_content, "t");
            if !shared.is_empty() {
                text_parts.push(shared);
            }
        }
    }

    // Extract raw sheet values
    for i in 1..=20 {
        let sheet_name = format!("xl/worksheets/sheet{}.xml", i);
        if let Ok(mut sheet_file) = archive.by_name(&sheet_name) {
            let mut xml_content = String::new();
            if sheet_file.read_to_string(&mut xml_content).is_ok() {
                let sheet_text = parse_xml_text_tags(&xml_content, "v");
                if !sheet_text.is_empty() {
                    text_parts.push(sheet_text);
                }
            }
        }
    }

    Ok(text_parts.join(" | "))
}

fn extract_pptx_text(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| format!("Cannot open PPTX: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid PPTX zip archive: {}", e))?;

    let mut text_parts = Vec::new();

    for i in 1..=100 {
        let slide_name = format!("ppt/slides/slide{}.xml", i);
        if let Ok(mut slide_file) = archive.by_name(&slide_name) {
            let mut xml_content = String::new();
            if slide_file.read_to_string(&mut xml_content).is_ok() {
                let slide_text = parse_xml_text_tags(&xml_content, "a:t");
                if !slide_text.is_empty() {
                    text_parts.push(format!("[Slide {}] {}", i, slide_text));
                }
            }
        }
    }

    Ok(text_parts.join("\n"))
}

fn extract_pdf_text(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| format!("Cannot open PDF: {}", e))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(|e| format!("Cannot read PDF: {}", e))?;

    let mut text_parts = Vec::new();
    let content = String::from_utf8_lossy(&buf);

    // Fast stream text extraction between (text) Tj and [(array)] TJ
    let mut in_parentheses = false;
    let mut current_str = String::new();

    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];
        if c == '(' && (i == 0 || chars[i - 1] != '\\') {
            in_parentheses = true;
            current_str.clear();
        } else if c == ')' && (i == 0 || chars[i - 1] != '\\') {
            in_parentheses = false;
            let clean = current_str.trim();
            if clean.len() > 1 && clean.chars().any(|ch| ch.is_alphabetic()) {
                text_parts.push(clean.to_string());
            }
            current_str.clear();
        } else if in_parentheses {
            current_str.push(c);
        }
        i += 1;
    }

    if text_parts.is_empty() {
        Ok(content.split_whitespace().filter(|w| w.len() > 3 && w.chars().all(|c| c.is_ascii_alphanumeric() || c.is_ascii_punctuation())).collect::<Vec<&str>>().join(" "))
    } else {
        Ok(text_parts.join(" "))
    }
}

fn parse_xml_text_tags(xml: &str, tag_name: &str) -> String {
    let open_tag = format!("<{}", tag_name);
    let close_tag = format!("</{}>", tag_name);

    let mut results = Vec::new();
    let mut pos = 0;

    while let Some(start) = xml[pos..].find(&open_tag) {
        let abs_start = pos + start;
        if let Some(tag_end) = xml[abs_start..].find('>') {
            let content_start = abs_start + tag_end + 1;
            if let Some(end) = xml[content_start..].find(&close_tag) {
                let text_chunk = &xml[content_start..content_start + end];
                let decoded = decode_xml_entities(text_chunk.trim());
                if !decoded.is_empty() {
                    results.push(decoded);
                }
                pos = content_start + end + close_tag.len();
            } else {
                pos = content_start;
            }
        } else {
            break;
        }
    }

    results.join(" ")
}

fn decode_xml_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}
