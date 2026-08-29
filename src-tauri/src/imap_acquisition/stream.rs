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
            if let Err(e) = client.select_mailbox(folder) {
                eprintln!("Failed to switch to mailbox {}: {}", folder, e);
                continue;
            }

            folders_acquired.push(folder.clone());
            let category = categorize_imap_folder(folder);

            let chunk_size: u32 = 25;
            let mut seq = 1;

            while seq <= *limit {
                if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                let chunk_end = (seq + chunk_size - 1).min(*limit);
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

                if fetch_res.is_err() {
                    for single_seq in seq..=chunk_end {
                        if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                            break;
                        }
                        if let Ok(raw) = client.fetch_raw_message(single_seq) {
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
                        } else {
                            errors += 1;
                            overall_processed += 1;
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
