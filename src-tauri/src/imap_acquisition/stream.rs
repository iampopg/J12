use std::sync::atomic::AtomicBool;
use super::client::ImapClient;
use super::{categorize_imap_folder, ImapAcquisitionResult, ImapConfig, StreamingMessage};

pub fn list_mailboxes(config: &ImapConfig) -> Result<Vec<String>, String> {
    let mut client = ImapClient::connect(config)?;
    client.list_mailboxes()
}

pub fn fetch_emails_streaming<F, G>(
    config: &ImapConfig,
    target_mailbox: Option<&str>,
    max_per_folder: Option<u32>,
    cancel_flag: &AtomicBool,
    mut on_folder_discovered: F,
    mut on_message: G,
) -> Result<ImapAcquisitionResult, String>
where
    F: FnMut(&str, u32, usize, usize, u32),
    G: FnMut(StreamingMessage) -> Result<(), String>,
{
    let mut client = ImapClient::connect(config)?;
    let all_folders = client.list_mailboxes()?;

    let folders_to_process = if let Some(single) = target_mailbox {
        if single.is_empty() || single.to_uppercase() == "ALL" {
            all_folders
        } else {
            vec![single.to_string()]
        }
    } else {
        all_folders
    };

    let total_folder_count = folders_to_process.len();
    let mut total_found = 0;
    let mut downloaded = 0;
    let mut errors = 0;
    let mut folders_acquired = Vec::new();

    let mut folder_counts = Vec::new();
    let mut overall_total_found: u32 = 0;

    for (idx, folder) in folders_to_process.iter().enumerate() {
        if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
        if total_folder_count > 1 && (folder.contains("All Mail") || folder.contains("Chats")) {
            continue;
        }
        if let Ok(count) = client.select_mailbox(folder) {
            let limit = if let Some(m) = max_per_folder { m.min(count) } else { count };
            overall_total_found += limit;
            folder_counts.push((folder.clone(), count, limit));
            on_folder_discovered(folder, count, idx + 1, total_folder_count, overall_total_found);
        }
    }

    let mut overall_processed: u32 = 0;

    for (f_idx, (folder, count, limit)) in folder_counts.iter().enumerate() {
        if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        total_found += count;

        if *count > 0 {
            let mut select_res = client.select_mailbox(folder);
            if select_res.is_err() {
                eprintln!("[IMAP] Socket disconnected prior to folder {}. Auto-reconnecting...", folder);
                if let Ok(mut new_client) = ImapClient::connect(config) {
                    select_res = new_client.select_mailbox(folder);
                    client = new_client;
                }
            }

            if let Err(e) = select_res {
                eprintln!("Failed to switch to mailbox {} after reconnect attempt: {}", folder, e);
                continue;
            }

            folders_acquired.push(folder.clone());
            let category = categorize_imap_folder(folder);

            let chunk_size: u32 = 50;
            let mut seq = 1;

            while seq <= *limit {
                if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                let chunk_end = (seq + chunk_size - 1).min(*limit);
                let mut chunk_attempt = 0;
                let mut chunk_success = false;

                while chunk_attempt < 3 && !chunk_success {
                    chunk_attempt += 1;
                    if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }

                    let fetch_res = client.fetch_chunk_messages(seq, chunk_end, |msg_seq, raw| {
                        downloaded += 1;
                        overall_processed += 1;
                        let msg = StreamingMessage {
                            folder_name: folder.clone(),
                            folder_category: category.clone(),
                            seq_id: msg_seq,
                            folder_total: *limit,
                            folder_index: f_idx + 1,
                            total_folders: total_folder_count,
                            overall_seq: overall_processed,
                            overall_total: overall_total_found,
                            raw_content: raw,
                        };
                        on_message(msg)
                    });

                    if fetch_res.is_ok() {
                        chunk_success = true;
                    } else {
                        eprintln!("[IMAP RECONNECT] Chunk {}-{} in [{}] interrupted (attempt {}/3). Reconnecting socket...", seq, chunk_end, folder, chunk_attempt);
                        std::thread::sleep(std::time::Duration::from_millis(1000));
                        if let Ok(mut new_client) = ImapClient::connect(config) {
                            let _ = new_client.select_mailbox(folder);
                            client = new_client;
                        }
                    }
                }

                if !chunk_success {
                    for single_seq in seq..=chunk_end {
                        if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                            break;
                        }
                        let mut single_res = client.fetch_raw_message(single_seq);
                        if single_res.is_err() {
                            if let Ok(mut new_client) = ImapClient::connect(config) {
                                let _ = new_client.select_mailbox(folder);
                                single_res = new_client.fetch_raw_message(single_seq);
                                client = new_client;
                            }
                        }
                        match single_res {
                            Ok(raw) => {
                                downloaded += 1;
                                overall_processed += 1;
                                let msg = StreamingMessage {
                                    folder_name: folder.clone(),
                                    folder_category: category.clone(),
                                    seq_id: single_seq,
                                    folder_total: *limit,
                                    folder_index: f_idx + 1,
                                    total_folders: total_folder_count,
                                    overall_seq: overall_processed,
                                    overall_total: overall_total_found,
                                    raw_content: raw,
                                };
                                let _ = on_message(msg);
                            }
                            Err(e) => {
                                eprintln!("Error fetching message {} from {}: {}", single_seq, folder, e);
                                errors += 1;
                                overall_processed += 1;
                            }
                        }
                    }
                }

                seq = chunk_end + 1;
            }
        }
    }

    Ok(ImapAcquisitionResult {
        total_found,
        downloaded,
        errors,
        folders_acquired,
        messages: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::sync::atomic::AtomicBool;
    use std::thread;

    #[test]
    fn test_imap_streaming_simulation() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
        let port = listener.local_addr().unwrap().port();

        let server_handle = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = stream.write_all(b"* OK Mock IMAP4rev1 Server Ready\r\n");
                let mut reader = BufReader::new(stream.try_clone().unwrap());

                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        break;
                    }
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.is_empty() {
                        continue;
                    }
                    let tag = parts[0];
                    let cmd = parts.get(1).map(|s| s.to_uppercase()).unwrap_or_default();

                    if cmd == "LOGIN" {
                        let _ = stream.write_all(format!("{} OK LOGIN completed\r\n", tag).as_bytes());
                    } else if cmd == "LIST" {
                        let _ = stream.write_all(b"* LIST () \"/\" \"INBOX\"\r\n");
                        let _ = stream.write_all(b"* LIST () \"/\" \"Sent\"\r\n");
                        let _ = stream.write_all(format!("{} OK LIST completed\r\n", tag).as_bytes());
                    } else if cmd == "SELECT" {
                        let mailbox = parts.get(2).unwrap_or(&"");
                        if mailbox.contains("INBOX") {
                            let _ = stream.write_all(b"* 2 EXISTS\r\n");
                        } else {
                            let _ = stream.write_all(b"* 0 EXISTS\r\n");
                        }
                        let _ = stream.write_all(format!("{} OK [READ-WRITE] SELECT completed\r\n", tag).as_bytes());
                    } else if cmd == "FETCH" {
                        let seq = parts.get(2).unwrap_or(&"1");
                        if seq.contains(':') {
                            let mut resp = String::new();
                            for i in 1..=2 {
                                let body = format!("From: sender@example.com\r\nSubject: Mock Subject {}\r\n\r\nTest Body {}", i, i);
                                resp.push_str(&format!("* {} FETCH (BODY[] {{{}}}\r\n{})\r\n", i, body.len(), body));
                            }
                            resp.push_str(&format!("{} OK FETCH completed\r\n", tag));
                            let _ = stream.write_all(resp.as_bytes());
                        } else {
                            let body = format!("From: sender@example.com\r\nSubject: Mock Subject {}\r\n\r\nTest Body {}", seq, seq);
                            let resp = format!("* {} FETCH (BODY[] {{{}}}\r\n{})\r\n{} OK FETCH completed\r\n", seq, body.len(), body, tag);
                            let _ = stream.write_all(resp.as_bytes());
                        }
                    } else if cmd == "LOGOUT" {
                        let _ = stream.write_all(format!("{} OK LOGOUT completed\r\n", tag).as_bytes());
                        break;
                    }
                }
            }
        });

        let config = ImapConfig {
            server: "127.0.0.1".to_string(),
            port,
            username: "test@example.com".to_string(),
            password: "password123".to_string(),
            auth_type: "password".to_string(),
            access_token: None,
            use_ssl: false,
            mailbox: "ALL".to_string(),
        };

        let cancel_flag = AtomicBool::new(false);
        let mut discovered = Vec::new();
        let mut received = Vec::new();

        let res = fetch_emails_streaming(
            &config,
            Some("ALL"),
            None,
            &cancel_flag,
            |folder, count, _idx, _total, _overall| {
                discovered.push((folder.to_string(), count));
            },
            |msg| {
                received.push(msg);
                Ok(())
            },
        ).expect("Streaming should succeed");

        assert_eq!(res.downloaded, 2);
        assert_eq!(received.len(), 2);
        assert!(received[0].raw_content.contains("Mock Subject 1"));
        assert!(received[1].raw_content.contains("Mock Subject 2"));

        let _ = server_handle.join();
    }
}
