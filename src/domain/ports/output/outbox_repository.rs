use crate::domain::entities::outbox::Outbox;
use async_trait::async_trait;
use sqlx::{Postgres, Transaction};

#[async_trait]
pub trait OutboxRepository: Send + Sync {
    async fn save_transactional(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        outbox: &Outbox,
    ) -> Result<(), OutboxError>;
}

#[derive(Debug)]
pub enum OutboxError {
    DatabaseError(String),
}

impl std::fmt::Display for OutboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutboxError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
        }
    }
}
