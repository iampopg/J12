use std::path::Path;
use std::process::Command;

pub fn extract_image_ocr(path: &Path, filename: &str) -> Result<String, String> {
    let lower_name = filename.to_lowercase();
    let is_image = lower_name.ends_with(".png")
        || lower_name.ends_with(".jpg")
        || lower_name.ends_with(".jpeg")
        || lower_name.ends_with(".tiff")
        || lower_name.ends_with(".bmp")
        || lower_name.ends_with(".webp")
        || lower_name.ends_with(".heic");

    if !is_image {
        return Ok("".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        // Try native Apple Vision OCR via osascript / swift command
        if let Ok(text) = macos_vision_ocr(path) {
            if !text.trim().is_empty() {
                return Ok(text);
            }
        }
    }

    // Try Tesseract if available on system
    if let Ok(text) = tesseract_ocr(path) {
        if !text.trim().is_empty() {
            return Ok(text);
        }
    }

    Ok("".to_string())
}

#[cfg(target_os = "macos")]
fn macos_vision_ocr(path: &Path) -> Result<String, String> {
    let path_str = path.to_str().ok_or("Invalid path string")?;
    let script = format!(
        r#"
        use framework "Vision"
        use framework "Foundation"
        set imgUrl to current application's NSURL's fileURLWithPath:"{}"
        set req to current application's VNRecognizeTextRequest's new()
        req's setRecognitionLevel:(current application's VNRequestTextRecognitionLevelAccurate)
        set handler to (current application's VNImageRequestHandler's alloc()'s initWithURL:imgUrl options:(current application's NSDictionary's dictionary()))
        set res to handler's performRequests:{{req}} |error|:(missing value)
        set allObservations to req's results()
        set outText to ""
        if allObservations is not missing value then
            repeat with obs in allObservations
                set candidates to (obs's topCandidates:1)
                if (count of candidates) > 0 then
                    set topCand to item 1 of candidates
                    set outText to outText & (topCand's string() as text) & linefeed
                end if
            end repeat
        end if
        return outText
        "#,
        path_str.replace("\"", "\\\"")
    );

    let output = Command::new("osascript")
        .arg("-l")
        .arg("AppleScript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("Apple Vision OCR error: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn tesseract_ocr(path: &Path) -> Result<String, String> {
    let path_str = path.to_str().ok_or("Invalid path string")?;
    let output = Command::new("tesseract")
        .arg(path_str)
        .arg("stdout")
        .arg("-l")
        .arg("eng")
        .output()
        .map_err(|e| format!("Tesseract execution error: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
