use crate::domain::value_objects::ZipCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Address {
    pub id: Uuid,
    pub street: String,
    pub number: Option<String>,
    pub complement: Option<String>,
    pub neighborhood: Option<String>,
    pub zip_code: ZipCode,
    pub ibge_code: Option<String>,
    pub state: String,
    pub state_uf: String,
    pub city: String,
    pub country: String,
    pub is_main: bool,
    pub tenant_id: Uuid,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

impl Address {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        street: String,
        number: Option<String>,
        complement: Option<String>,
        neighborhood: Option<String>,
        zip_code: ZipCode,
        ibge_code: Option<String>,
        state: String,
        state_uf: String,
        city: String,
        country: String,
        is_main: bool,
        tenant_id: Uuid,
        created_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            street,
            number,
            complement,
            neighborhood,
            zip_code,
            ibge_code,
            state,
            state_uf,
            city,
            country,
            is_main,
            tenant_id,
            created_by,
            created_at: Utc::now(),
        }
    }
}
