use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEvent {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub aggregate_id: Uuid,     // ID da Pessoa (no nosso caso)
    pub aggregate_type: String, // "PERSON"
    pub event_type: String,     // "PERSON_CREATED", "PERSON_UPDATED"
    pub payload: Value,         // O JSON do evento
    pub occurred_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>, // Quando foi enviado ao Kafka
}
