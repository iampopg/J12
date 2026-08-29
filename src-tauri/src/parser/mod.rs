pub mod types;
pub mod mime;
pub mod headers;
pub mod eml;
pub mod mbox;

pub use types::*;
pub use mime::*;
pub use headers::*;
pub use eml::*;
pub use mbox::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn temp_file(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    #[test]
    fn test_eml_standard_headers() {
        let content = r#"From: "John Doe" <john@example.com>
To: "Jane Smith" <jane@example.com>
Subject: Test Subject
Date: Mon, 15 Jan 2024 09:15:00 -0800
Message-ID: <test-001@example.com>
MIME-Version: 1.0
Content-Type: text/plain; charset="utf-8"

Test body content.
"#;
        let f = temp_file(content);
        let emails = parse_eml(f.path()).unwrap();
        assert_eq!(emails.len(), 1);
        assert_eq!(emails[0].from_addr, "john@example.com");
        assert_eq!(emails[0].from_display, Some("John Doe".to_string()));
        assert_eq!(emails[0].to_addrs.len(), 1);
        assert_eq!(emails[0].subject, Some("Test Subject".to_string()));
    }

    #[test]
    fn test_eml_missing_message_id() {
        let content = r#"From: sender@example.com
To: recipient@example.com
Subject: No Message-ID

Body
"#;
        let f = temp_file(content);
        let emails = parse_eml(f.path()).unwrap();
        assert_eq!(emails.len(), 1);
        assert!(!emails[0].message_id.is_empty());
        assert!(emails[0].warnings.iter().any(|w| w.contains("Generated")));
    }

    #[test]
    fn test_eml_missing_from() {
        let content = r#"To: recipient@example.com
Subject: No From

Body
"#;
        let f = temp_file(content);
        let emails = parse_eml(f.path()).unwrap();
        assert_eq!(emails.len(), 1);
        assert_eq!(emails[0].from_addr, "unknown@unknown");
    }

    #[test]
    fn test_eml_multiple_to() {
        let content = r#"From: sender@example.com
To: user1@example.com, user2@example.com, user3@example.com
Subject: Multiple Recipients

Body
"#;
        let f = temp_file(content);
        let emails = parse_eml(f.path()).unwrap();
        assert_eq!(emails.len(), 1);
        assert_eq!(emails[0].to_addrs.len(), 3);
    }

    #[test]
    fn test_eml_mime_encoded_subject() {
        let content = r#"From: sender@example.com
To: recipient@example.com
Subject: =?UTF-8?B?SGVsbG8gV29ybGQ=?=
Date: Mon, 15 Jan 2024 09:15:00 -0800
Message-ID: <encoded@example.com>

Body
"#;
        let f = temp_file(content);
        let emails = parse_eml(f.path()).unwrap();
        assert_eq!(emails.len(), 1);
        assert_eq!(emails[0].subject, Some("Hello World".to_string()));
    }

    #[test]
    fn test_eml_multipart_with_attachments() {
        let content = r#"From: sender@example.com
To: recipient@example.com
Subject: With Attachment
Date: Mon, 15 Jan 2024 09:15:00 -0800
Message-ID: <attach@example.com>
MIME-Version: 1.0
Content-Type: multipart/mixed; boundary="boundary123"

--boundary123
Content-Type: text/plain; charset="utf-8"

Body text

--boundary123
Content-Type: application/pdf; name="doc.pdf"
Content-Transfer-Encoding: base64
Content-Disposition: attachment; filename="doc.pdf"

JVBERi0xLjQKMSAwIG9iago8PAovVHlwZSAvQ2F0YWxvZwovUGFnZXMgMiAwIFIKPj4KZW5k
b2JqCg==

--boundary123--
"#;
        let f = temp_file(content);
        let emails = parse_eml(f.path()).unwrap();
        assert_eq!(emails.len(), 1);
        assert!(emails[0].body_text.is_some());
    }

    #[test]
    fn test_eml_folded_headers() {
        let content = r#"From: sender@example.com
To: recipient@example.com
Subject: This is a very long subject line that should be
 folded across multiple lines properly
Date: Mon, 15 Jan 2024 09:15:00 -0800
Message-ID: <folded@example.com>

Body
"#;
        let f = temp_file(content);
        let emails = parse_eml(f.path()).unwrap();
        assert_eq!(emails.len(), 1);
        assert!(emails[0].subject.as_ref().unwrap().contains("folded"));
    }

    #[test]
    fn test_mbox_multiple_messages() {
        let content = r#"From sender@example.com Mon Jan 15 09:15:00 2024
From: sender@example.com
To: recipient@example.com
Subject: First Message
Date: Mon, 15 Jan 2024 09:15:00 -0800
Message-ID: <mbox-001@example.com>

First body.

From sender@example.com Mon Jan 15 10:00:00 2024
From: sender@example.com
To: other@example.com
Subject: Second Message
Date: Mon, 15 Jan 2024 10:00:00 -0800
Message-ID: <mbox-002@example.com>

Second body.
"#;
        let f = temp_file(content);
        let emails = parse_mbox(f.path()).unwrap();
        assert_eq!(emails.len(), 2);
        assert_eq!(emails[0].subject, Some("First Message".to_string()));
        assert_eq!(emails[1].subject, Some("Second Message".to_string()));
    }

    #[test]
    fn test_mbox_corrupted_recovery() {
        let content = r#"From sender@example.com Mon Jan 15 09:15:00 2024
From: sender@example.com
To: recipient@example.com
Subject: Valid Message
Date: Mon, 15 Jan 2024 09:15:00 -0800
Message-ID: <valid@example.com>

Valid body.

This is corrupted data without proper headers
From another line that looks like separator
"#;
        let f = temp_file(content);
        let emails = parse_mbox(f.path()).unwrap();
        assert!(emails.len() >= 1);
        assert_eq!(emails[0].subject, Some("Valid Message".to_string()));
    }

    #[test]
    fn test_eml_exchange_x_headers() {
        let content = r#"From: sender@example.com
To: recipient@example.com
Subject: Exchange Headers
Date: Mon, 15 Jan 2024 09:15:00 -0800
Message-ID: <xheaders@example.com>
X-From: "Doe, John" </O=ENRON/OU=NA/CN=RECIPIENTS/CN=JDOE>
X-To: "Smith, Jane" </O=ENRON/OU=NA/CN=RECIPIENTS/CN=JSMITH>
X-Folder: \Inbox

Body
"#;
        let f = temp_file(content);
        let emails = parse_eml(f.path()).unwrap();
        assert_eq!(emails.len(), 1);
        assert!(emails[0].from_display.as_ref().map(|s| s.contains("Doe")).unwrap_or(false));
        assert!(!emails[0].to_display_names.is_empty());
    }

    #[test]
    fn test_eml_empty_body() {
        let content = r#"From: sender@example.com
To: recipient@example.com
Subject: Empty Body
Date: Mon, 15 Jan 2024 09:15:00 -0800
Message-ID: <empty@example.com>
Content-Type: text/plain

"#;
        let f = temp_file(content);
        let emails = parse_eml(f.path()).unwrap();
        assert_eq!(emails.len(), 1);
    }

    #[test]
    fn test_eml_nested_message() {
        let content = r#"From: sender@example.com
To: recipient@example.com
Subject: Forwarded Message
Date: Mon, 15 Jan 2024 09:15:00 -0800
Message-ID: <nested@example.com>
MIME-Version: 1.0
Content-Type: multipart/mixed; boundary="outer"

--outer
Content-Type: text/plain

Forwarded message below

--outer
Content-Type: message/rfc822

From: original@example.com
To: recipient@example.com
Subject: Original Message
Date: Mon, 15 Jan 2024 08:00:00 -0800
Message-ID: <original@example.com>

Original body

--outer--
"#;
        let f = temp_file(content);
        let emails = parse_eml(f.path()).unwrap();
        assert_eq!(emails.len(), 1);
    }
}
