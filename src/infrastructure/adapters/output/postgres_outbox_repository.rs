use crate::domain::entities::outbox::Outbox;
use crate::domain::ports::output::outbox_repository::{OutboxError, OutboxRepository};
use async_trait::async_trait;
use sqlx::{Postgres, Transaction};

pub struct PostgresOutboxRepository;

impl PostgresOutboxRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl OutboxRepository for PostgresOutboxRepository {
    async fn save_transactional(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        outbox: &Outbox,
    ) -> Result<(), OutboxError> {
        sqlx::query(
            r#"
            INSERT INTO outboxes (
                id, tenant_id, aggregate_id, aggregate_type, 
                event_type, payload, created_by, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(outbox.id)
        .bind(outbox.tenant_id)
        .bind(outbox.aggregate_id)
        .bind(outbox.aggregate_type.to_string())
        .bind(outbox.event_type.to_string())
        .bind(&outbox.payload)
        .bind(outbox.created_by)
        .bind(outbox.created_at)
        .execute(&mut **tx)
        .await
        .map_err(|e| OutboxError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
