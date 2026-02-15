use crate::domain::entities::person::PersonData;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreatePersonRequest {
    pub name: String,
    pub data: PersonData, // Reutilizamos o Enum de domínio ou criamos um específico
}
