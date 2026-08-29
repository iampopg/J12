use chrono::Timelike;
use tauri::State;
use crate::AppState;
use super::types::{TimelineEvent, TimelineInterpretation, TimelineAnomaly};
use super::context::ai_get_timeline;

// === ENGINE 4: TIMELINE RECONSTRUCTION ===

pub fn analyze_timeline(events: &[TimelineEvent]) -> TimelineInterpretation {
    let mut anomalies = Vec::new();
    let mut clock_skew_detected = false;
    let mut timestamp_reversals = Vec::new();
    
    for window in events.windows(2) {
        let current = &window[0];
        let next = &window[1];
        
        if current.timestamp > next.timestamp {
            timestamp_reversals.push((current.timestamp.clone(), next.timestamp.clone()));
            anomalies.push(TimelineAnomaly {
                event_id: next.id.clone(),
                anomaly_type: "timestamp_reversal".to_string(),
                description: format!("Event at {} occurs before event at {}", next.timestamp, current.timestamp),
                severity: "medium".to_string(),
                timestamp: next.timestamp.clone(),
            });
        }
    }
    
    for event in events {
        if let Ok(dt) = event.timestamp.parse::<chrono::DateTime<chrono::Utc>>() {
            let hour = dt.hour();
            if hour < 6 || hour > 22 {
                anomalies.push(TimelineAnomaly {
                    event_id: event.id.clone(),
                    anomaly_type: "unusual_hour".to_string(),
                    description: format!("Event at {} occurred outside typical business hours (hour: {})", event.timestamp, hour),
                    severity: "low".to_string(),
                    timestamp: event.timestamp.clone(),
                });
            }
        }
    }
    
    if events.len() >= 3 {
        for window in events.windows(3) {
            if let (Ok(t1), Ok(t3)) = (
                window[0].timestamp.parse::<chrono::DateTime<chrono::Utc>>(),
                window[2].timestamp.parse::<chrono::DateTime<chrono::Utc>>()
            ) {
                let duration = t3.signed_duration_since(t1);
                if duration.num_minutes() < 5 {
                    anomalies.push(TimelineAnomaly {
                        event_id: window[1].id.clone(),
                        anomaly_type: "burst_activity".to_string(),
                        description: format!("Three events within {} minutes", duration.num_minutes()),
                        severity: "medium".to_string(),
                        timestamp: window[1].timestamp.clone(),
                    });
                }
            }
        }
    }
    
    if !timestamp_reversals.is_empty() {
        clock_skew_detected = true;
    }
    
    let narrative = generate_timeline_narrative(events, &anomalies, clock_skew_detected);
    
    TimelineInterpretation {
        events: events.to_vec(),
        anomalies,
        narrative,
        clock_skew_detected,
        timestamp_reversals,
    }
}

fn generate_timeline_narrative(events: &[TimelineEvent], anomalies: &[TimelineAnomaly], clock_skew: bool) -> String {
    let mut narrative = String::from("## Timeline Analysis\n\n");
    
    if events.is_empty() {
        narrative.push_str("No timeline events found.\n");
        return narrative;
    }
    
    narrative.push_str(&format!("**Total Events:** {}\n", events.len()));
    narrative.push_str(&format!("**Anomalies Detected:** {}\n\n", anomalies.len()));
    
    if clock_skew {
        narrative.push_str("⚠️ **Clock Skew Detected:** Timestamp reversals found. Server clocks may be unsynchronized or timestamps may be forged.\n\n");
    }
    
    if !anomalies.is_empty() {
        narrative.push_str("**Key Anomalies:**\n");
        for anomaly in anomalies.iter().take(5) {
            narrative.push_str(&format!("- [{}] {}: {}\n", anomaly.severity.to_uppercase(), anomaly.anomaly_type, anomaly.description));
        }
    }
    
    narrative
}

#[tauri::command]
pub async fn ai_analyze_timeline(state: State<'_, AppState>, case_id: String, limit: Option<i64>) -> Result<TimelineInterpretation, String> {
    let events = ai_get_timeline(state, case_id, limit).await?;
    Ok(analyze_timeline(&events))
}
