use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AggregateType {
    Person,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventType {
    PersonCreated,
    PersonUpdated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outbox {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub aggregate_id: Uuid,
    pub aggregate_type: AggregateType,
    pub event_type: EventType,
    pub payload: Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

impl Outbox {
    pub fn new(
        tenant_id: Uuid,
        aggregate_id: Uuid,
        aggregate_type: AggregateType,
        event_type: EventType,
        payload: Value,
        created_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            aggregate_id,
            aggregate_type,
            event_type,
            payload,
            created_by,
            created_at: Utc::now(),
            processed_at: None,
        }
    }
}
