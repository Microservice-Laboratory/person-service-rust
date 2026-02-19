use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::outbox::OutboxEvent;
use crate::domain::value_objects::{Cnpj, Cpf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonData {
    Individual {
        tax_id: Cpf,
    },
    LegalEntity {
        business_tax_id: Cnpj,
        trade_name: Option<String>,
    },
    Foreign {
        passport_number: String,
        country_code: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Person {
    pub id: Uuid,
    pub name: String,
    pub tenant_id: Uuid,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub data: PersonData,
}

impl Person {
    pub fn to_outbox_event(&self, event_type: &str) -> OutboxEvent {
        OutboxEvent {
            id: Uuid::new_v4(),
            tenant_id: self.tenant_id,
            aggregate_id: self.id,
            aggregate_type: "PERSON".to_string(),
            event_type: event_type.to_string(),
            payload: serde_json::to_value(self).unwrap_or_default(),
            occurred_at: Utc::now(),
            processed_at: None,
        }
    }
}
