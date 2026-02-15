use crate::domain::entities::person::Person;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait PersonRepository: Send + Sync {
    // Note que o tenant_id é explícito para garantir a segurança (Multi-tenancy)
    async fn save(&self, person: &Person) -> Result<(), PersonError>;

    async fn find_by_id(&self, id: Uuid, tenant_id: Uuid) -> Result<Option<Person>, PersonError>;

    async fn exists_by_tax_id(&self, tax_id: &str, tenant_id: Uuid) -> Result<bool, PersonError>;
}

#[derive(Debug)]
pub enum PersonError {
    DatabaseError(String),
    Conflict(String),
    NotFound,
}

impl std::fmt::Display for PersonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersonError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            PersonError::Conflict(msg) => write!(f, "Conflict Error: {}", msg),
            PersonError::NotFound => write!(f, "Person Not Found"),
        }
    }
}
