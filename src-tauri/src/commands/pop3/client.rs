use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;

#[derive(Debug, Clone)]
pub struct Pop3Config {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_ssl: bool,
}

#[derive(Debug, Clone)]
pub struct Pop3AcquisitionResult {
    pub total_found: u32,
    pub downloaded: u32,
    pub errors: u32,
}

pub enum Pop3Stream {
    Tls(native_tls::TlsStream<TcpStream>),
    Plain(TcpStream),
}

impl Read for Pop3Stream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Pop3Stream::Tls(s) => s.read(buf),
            Pop3Stream::Plain(s) => s.read(buf),
        }
    }
}

impl Write for Pop3Stream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Pop3Stream::Tls(s) => s.write(buf),
            Pop3Stream::Plain(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Pop3Stream::Tls(s) => s.flush(),
            Pop3Stream::Plain(s) => s.flush(),
        }
    }
}

pub struct Pop3Client {
    stream: BufReader<Pop3Stream>,
}

impl Pop3Client {
    pub fn connect(config: &Pop3Config) -> Result<Self, String> {
        let addr = format!("{}:{}", config.server, config.port);
        let tcp_stream = TcpStream::connect(&addr)
            .map_err(|e| format!("Failed to connect to {}: {}", addr, e))?;
        let _ = tcp_stream.set_read_timeout(Some(std::time::Duration::from_secs(15)));
        let _ = tcp_stream.set_write_timeout(Some(std::time::Duration::from_secs(15)));

        let stream = if config.use_ssl || config.port == 995 {
            let connector = native_tls::TlsConnector::new()
                .map_err(|e| format!("TLS init error: {}", e))?;
            let tls_stream = connector.connect(&config.server, tcp_stream)
                .map_err(|e| format!("TLS handshake error: {}", e))?;
            Pop3Stream::Tls(tls_stream)
        } else {
            Pop3Stream::Plain(tcp_stream)
        };

        let mut client = Pop3Client {
            stream: BufReader::new(stream),
        };

        let mut greeting = String::new();
        client.stream.read_line(&mut greeting)
            .map_err(|e| format!("Failed reading greeting: {}", e))?;

        if !greeting.contains("+OK") {
            return Err(format!("POP3 server rejected connection: {}", greeting.trim()));
        }

        client.login(&config.username, &config.password)?;

        Ok(client)
    }

    pub fn send_command(&mut self, cmd: &str) -> Result<(String, Vec<String>), String> {
        let full_cmd = format!("{}\r\n", cmd);
        self.stream.get_mut().write_all(full_cmd.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;
        self.stream.get_mut().flush()
            .map_err(|e| format!("Flush error: {}", e))?;

        let mut first_line = String::new();
        self.stream.read_line(&mut first_line)
            .map_err(|e| format!("Read error: {}", e))?;

        if !first_line.contains("+OK") {
            return Err(format!("POP3 error: {}", first_line.trim()));
        }

        if cmd.starts_with("STAT") || cmd.starts_with("QUIT") {
            return Ok((first_line, vec![]));
        }

        let mut lines = Vec::new();
        loop {
            let mut line = String::new();
            let n = self.stream.read_line(&mut line)
                .map_err(|e| format!("Read error: {}", e))?;
            if n == 0 || line == ".\r\n" || line == ".\n" {
                break;
            }
            lines.push(line.trim_end_matches(&['\r', '\n'][..]).to_string());
        }

        Ok((first_line, lines))
    }

    fn login(&mut self, user: &str, pass: &str) -> Result<(), String> {
        let clean_user = user.trim();
        let mut clean_pass = pass.trim().trim_matches(&['\\', '"', '\'', ' '][..]).trim().to_string();
        
        let no_spaces = clean_pass.replace(' ', "");
        if no_spaces.len() == 16 || clean_user.to_lowercase().contains("gmail") {
            clean_pass = no_spaces;
        }

        let _ = self.send_command(&format!("USER {}", clean_user))?;
        match self.send_command(&format!("PASS {}", clean_pass)) {
            Ok(_) => Ok(()),
            Err(e) => {
                let pass_no_spaces = clean_pass.replace(' ', "");
                if pass_no_spaces != clean_pass && !pass_no_spaces.is_empty() {
                    let _ = self.send_command(&format!("USER {}", clean_user));
                    if self.send_command(&format!("PASS {}", pass_no_spaces)).is_ok() {
                        return Ok(());
                    }
                }
                Err(format!("POP3 Authentication failed. Check your password or use an App Password: {}", e))
            }
        }
    }

    pub fn get_message_count(&mut self) -> Result<u32, String> {
        let (line, _) = self.send_command("STAT")?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            parts[1].parse::<u32>().map_err(|e| format!("Parse error: {}", e))
        } else {
            Ok(0)
        }
    }

    pub fn fetch_raw_message(&mut self, msg_num: u32) -> Result<String, String> {
        let (_, lines) = self.send_command(&format!("RETR {}", msg_num))?;
        Ok(lines.join("\r\n"))
    }
}
