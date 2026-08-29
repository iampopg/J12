use std::collections::HashMap;
use tauri::State;
use crate::AppState;
use super::types::{GraphAnalysis, EntityCentrality, GraphAnomaly, EntityData};

// === ENGINE 7: GRAPH ANALYST ===

pub fn analyze_graph(entities: &[EntityData], edges: &[(String, String, i64)]) -> GraphAnalysis {
    let mut central_entities = Vec::new();
    let mut anomalies = Vec::new();
    
    let mut connection_counts: HashMap<String, usize> = HashMap::new();
    for (from, to, _) in edges {
        *connection_counts.entry(from.clone()).or_insert(0) += 1;
        *connection_counts.entry(to.clone()).or_insert(0) += 1;
    }
    
    for entity in entities {
        let count = connection_counts.get(&entity.email_address).copied().unwrap_or(0);
        let total_possible = entities.len().saturating_sub(1);
        let centrality = if total_possible > 0 {
            count as f64 / total_possible as f64
        } else {
            0.0
        };
        
        central_entities.push(EntityCentrality {
            entity_id: entity.id.clone(),
            email_address: entity.email_address.clone(),
            centrality_score: centrality,
            connection_count: count,
        });
    }
    
    central_entities.sort_by(|a, b| b.centrality_score.partial_cmp(&a.centrality_score).unwrap());
    
    for entity in entities {
        let count = connection_counts.get(&entity.email_address).copied().unwrap_or(0);
        if count == 0 {
            anomalies.push(GraphAnomaly {
                anomaly_type: "isolated_entity".to_string(),
                description: format!("{} has no communication connections", entity.email_address),
                entities_involved: vec![entity.id.clone()],
                severity: "low".to_string(),
            });
        }
    }
    
    for central in central_entities.iter().take(3) {
        if central.centrality_score > 0.5 {
            anomalies.push(GraphAnomaly {
                anomaly_type: "central_hub".to_string(),
                description: format!("{} is a communication hub (centrality: {:.2})", central.email_address, central.centrality_score),
                entities_involved: vec![central.entity_id.clone()],
                severity: "medium".to_string(),
            });
        }
    }
    
    let mut recommendations = Vec::new();
    if !central_entities.is_empty() {
        recommendations.push(format!("Focus investigation on top 3 central entities: {}", 
            central_entities.iter().take(3).map(|e| &e.email_address).cloned().collect::<Vec<_>>().join(", ")));
    }
    if !anomalies.is_empty() {
        recommendations.push(format!("Review {} graph anomalies for suspicious patterns", anomalies.len()));
    }
    
    GraphAnalysis {
        central_entities,
        communities: Vec::new(),
        anomalies,
        recommendations,
    }
}

#[tauri::command]
pub async fn ai_analyze_graph(state: State<'_, AppState>, case_id: String, max_nodes: Option<i64>) -> Result<GraphAnalysis, String> {
    let db = state.db.lock().await;
    
    let mut entity_stmt = db.conn.prepare(
        "SELECT id, email_address, display_name, sent_count, received_count, first_seen, last_seen FROM entities WHERE case_id = ?1"
    ).map_err(|e| e.to_string())?;
    
    let entities = entity_stmt.query_map([&case_id], |row| {
        Ok(EntityData {
            id: row.get(0)?,
            email_address: row.get(1)?,
            display_name: row.get(2)?,
            sent_count: row.get(3)?,
            received_count: row.get(4)?,
            first_seen: row.get(5)?,
            last_seen: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    let max = max_nodes.unwrap_or(500);
    let mut edge_stmt = db.conn.prepare(
        "SELECT from_entity, to_entity, message_count FROM communication_edges WHERE case_id = ?1 LIMIT ?2"
    ).map_err(|e| e.to_string())?;
    
    let edges = edge_stmt.query_map(rusqlite::params![&case_id, max], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    
    Ok(analyze_graph(&entities, &edges))
}
