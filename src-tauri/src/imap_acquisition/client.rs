use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use native_tls::TlsConnector;
use super::ImapConfig;

pub enum ImapStream {
    Tls(native_tls::TlsStream<TcpStream>),
    Plain(TcpStream),
}

impl Read for ImapStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            ImapStream::Tls(s) => s.read(buf),
            ImapStream::Plain(s) => s.read(buf),
        }
    }
}

impl Write for ImapStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            ImapStream::Tls(s) => s.write(buf),
            ImapStream::Plain(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            ImapStream::Tls(s) => s.flush(),
            ImapStream::Plain(s) => s.flush(),
        }
    }
}

pub struct ImapClient {
    stream: BufReader<ImapStream>,
    tag_counter: u32,
}

impl ImapClient {
    pub fn connect(config: &ImapConfig) -> Result<Self, String> {
        let addr = format!("{}:{}", config.server, config.port);
        let tcp_stream = TcpStream::connect(&addr).map_err(|e| format!("Failed to connect to {}: {}", addr, e))?;
        let _ = tcp_stream.set_read_timeout(Some(std::time::Duration::from_secs(60)));
        let _ = tcp_stream.set_write_timeout(Some(std::time::Duration::from_secs(60)));

        let stream = if config.use_ssl || config.port == 993 {
            let connector = TlsConnector::new().map_err(|e| format!("TLS init error: {}", e))?;
            let tls_stream = connector.connect(&config.server, tcp_stream).map_err(|e| format!("TLS handshake error: {}", e))?;
            ImapStream::Tls(tls_stream)
        } else {
            ImapStream::Plain(tcp_stream)
        };

        let mut client = ImapClient {
            stream: BufReader::new(stream),
            tag_counter: 1,
        };

        let mut banner = String::new();
        client.stream.read_line(&mut banner).map_err(|e| format!("Failed reading greeting: {}", e))?;
        
        if config.auth_type == "oauth2" {
            let token = config.access_token.as_deref().unwrap_or(&config.password);
            client.authenticate_xoauth2(&config.username, token)?;
        } else {
            client.login(&config.username, &config.password)?;
        }

        Ok(client)
    }

    fn next_tag(&mut self) -> String {
        let tag = format!("A{:04}", self.tag_counter);
        self.tag_counter += 1;
        tag
    }

    pub fn authenticate_xoauth2(&mut self, user: &str, access_token: &str) -> Result<(), String> {
        let xoauth_payload = super::oauth::generate_xoauth2_string(user, access_token);
        let cmd = format!("AUTHENTICATE XOAUTH2 {}", xoauth_payload);
        let (status, _) = self.send_command(&cmd).map_err(|e| {
            format!("SASL XOAUTH2 Authentication failed. Verify your OAuth2 Access Token: {}", e)
        })?;
        if status.contains("OK") {
            Ok(())
        } else {
            Err(format!("SASL XOAUTH2 Authentication rejected: {}", status))
        }
    }

    pub fn send_command(&mut self, cmd: &str) -> Result<(String, Vec<String>), String> {
        let tag = self.next_tag();
        let full_cmd = format!("{} {}\r\n", tag, cmd);
        self.stream.get_mut().write_all(full_cmd.as_bytes()).map_err(|e| format!("Write command error: {}", e))?;
        self.stream.get_mut().flush().map_err(|e| format!("Flush error: {}", e))?;

        let mut responses = Vec::new();
        loop {
            let mut line = String::new();
            let n = self.stream.read_line(&mut line).map_err(|e| format!("Read error: {}", e))?;
            if n == 0 {
                return Err("Connection closed unexpectedly by IMAP server".to_string());
            }

            let trimmed = line.trim_end_matches(&['\r', '\n'][..]).to_string();
            if trimmed.starts_with(&tag) {
                if trimmed.contains("OK") {
                    return Ok((trimmed, responses));
                } else {
                    return Err(format!("IMAP error: {}", trimmed));
                }
            } else {
                responses.push(trimmed);
            }
        }
    }

    fn login(&mut self, user: &str, pass: &str) -> Result<(), String> {
        let clean_user = user.trim().replace('"', "\\\"");
        let mut clean_pass = pass.trim().trim_matches(&['\\', '"', '\'', ' '][..]).trim().to_string();
        
        let no_spaces = clean_pass.replace(' ', "");
        if no_spaces.len() == 16 || user.to_lowercase().contains("gmail") {
            clean_pass = no_spaces;
        }

        let cmd = format!("LOGIN \"{}\" \"{}\"", clean_user, clean_pass.replace('"', "\\\""));
        self.send_command(&cmd).map_err(|e| format!("IMAP Authentication failed. Check your password or use an App Password: {}", e))?;
        Ok(())
    }

    pub fn list_mailboxes(&mut self) -> Result<Vec<String>, String> {
        let (_, lines) = self.send_command("LIST \"\" \"*\"")?;
        let mut boxes = Vec::new();
        for line in lines {
            if line.to_uppercase().starts_with("* LIST") {
                if let Some(last_quote) = line.rfind('"') {
                    if let Some(first_quote) = line[..last_quote].rfind('"') {
                        let name = line[first_quote+1..last_quote].to_string();
                        if !name.is_empty() && !boxes.contains(&name) {
                            boxes.push(name);
                            continue;
                        }
                    }
                }
                if let Some(last_space) = line.rfind(' ') {
                    let folder = line[last_space+1..].trim().to_string();
                    if !folder.is_empty() && !boxes.contains(&folder) {
                        boxes.push(folder);
                    }
                }
            }
        }
        if boxes.is_empty() {
            boxes.push("INBOX".to_string());
        }
        Ok(boxes)
    }

    pub fn select_mailbox(&mut self, mailbox: &str) -> Result<u32, String> {
        let cmd = format!("SELECT \"{}\"", mailbox);
        let (_, lines) = self.send_command(&cmd)?;
        let mut exists_count = 0;
        for line in lines {
            let upper = line.to_uppercase();
            if upper.starts_with("* ") && upper.contains("EXISTS") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(num) = parts[1].parse::<u32>() {
                        exists_count = num;
                    }
                }
            }
        }
        Ok(exists_count)
    }

    pub fn fetch_chunk_messages<F>(
        &mut self,
        start_seq: u32,
        end_seq: u32,
        mut on_raw_msg: F,
    ) -> Result<u32, String>
    where
        F: FnMut(u32, String) -> Result<(), String>,
    {
        let tag = self.next_tag();
        let cmd = format!("{} FETCH {}:{} (BODY.PEEK[])\r\n", tag, start_seq, end_seq);
        self.stream.get_mut().write_all(cmd.as_bytes()).map_err(|e| e.to_string())?;
        self.stream.get_mut().flush().map_err(|e| e.to_string())?;

        let mut received = 0;
        loop {
            let mut line = String::new();
            let n = self.stream.read_line(&mut line).map_err(|e| e.to_string())?;
            if n == 0 {
                return Err("Connection closed during chunk fetch".to_string());
            }

            if line.starts_with(&tag) {
                break;
            }

            if line.contains('{') && line.contains('}') {
                let seq = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(start_seq + received);

                let byte_count = if let Some(open) = line.rfind('{') {
                    if let Some(close) = line[open..].find('}') {
                        line[open + 1..open + close].parse::<usize>().unwrap_or(0)
                    } else {
                        0
                    }
                } else {
                    0
                };

                let mut body_bytes = vec![0u8; byte_count];
                if byte_count > 0 {
                    self.stream.read_exact(&mut body_bytes).map_err(|e| e.to_string())?;
                }

                let mut closing_line = String::new();
                let _ = self.stream.read_line(&mut closing_line);

                let raw_str = String::from_utf8(body_bytes.clone())
                    .unwrap_or_else(|_| String::from_utf8_lossy(&body_bytes).to_string());

                received += 1;
                let _ = on_raw_msg(seq, raw_str);
            }
        }

        Ok(received)
    }

    pub fn fetch_raw_message(&mut self, seq_id: u32) -> Result<String, String> {
        let tag = self.next_tag();
        let cmd = format!("{} FETCH {} (BODY.PEEK[])\r\n", tag, seq_id);
        self.stream.get_mut().write_all(cmd.as_bytes()).map_err(|e| e.to_string())?;
        self.stream.get_mut().flush().map_err(|e| e.to_string())?;

        let mut first_line = String::new();
        loop {
            let n = self.stream.read_line(&mut first_line).map_err(|e| e.to_string())?;
            if n == 0 {
                return Err("Connection closed while waiting for FETCH response".to_string());
            }
            if first_line.contains('{') || first_line.starts_with(&tag) {
                break;
            }
            first_line.clear();
        }

        if first_line.starts_with(&tag) {
            return Err(format!("Fetch response error: {}", first_line));
        }

        let byte_count = if let Some(open_brace) = first_line.rfind('{') {
            if let Some(close_brace) = first_line[open_brace..].find('}') {
                let num_str = &first_line[open_brace+1..open_brace+close_brace];
                num_str.parse::<usize>().unwrap_or(0)
            } else { 0 }
        } else { 0 };

        let mut body_bytes = vec![0u8; byte_count];
        if byte_count > 0 {
            self.stream.read_exact(&mut body_bytes).map_err(|e| e.to_string())?;
        }

        loop {
            let mut line = String::new();
            let n = self.stream.read_line(&mut line).map_err(|e| e.to_string())?;
            if n == 0 || line.starts_with(&tag) {
                break;
            }
        }

        String::from_utf8(body_bytes.clone()).or_else(|_| Ok(String::from_utf8_lossy(&body_bytes).to_string()))
    }
}
