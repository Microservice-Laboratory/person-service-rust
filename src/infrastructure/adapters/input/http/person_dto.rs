use crate::domain::entities::person::PersonData;
use crate::domain::value_objects::{Cnpj, Cpf};
use serde::Deserialize;
use std::convert::TryInto;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DtoError {
    #[error("Invalid data: {0}")]
    ValidationError(String),
}

#[derive(Deserialize)]
pub enum PersonDataDTO {
    Individual {
        tax_id: String,
    },
    LegalEntity {
        business_tax_id: String,
        trade_name: Option<String>,
    },
    Foreign {
        passport_number: String,
        country_code: String,
    },
}

#[derive(Deserialize)]
pub struct CreatePersonRequest {
    pub name: String,
    pub data: PersonDataDTO,
}

impl TryInto<PersonData> for PersonDataDTO {
    type Error = DtoError;

    fn try_into(self) -> Result<PersonData, Self::Error> {
        match self {
            PersonDataDTO::Individual { tax_id } => {
                let cpf =
                    Cpf::new(&tax_id).map_err(|e| DtoError::ValidationError(e.to_string()))?;
                Ok(PersonData::Individual { tax_id: cpf })
            }
            PersonDataDTO::LegalEntity {
                business_tax_id,
                trade_name,
            } => {
                let cnpj = Cnpj::new(&business_tax_id)
                    .map_err(|e| DtoError::ValidationError(e.to_string()))?;
                Ok(PersonData::LegalEntity {
                    business_tax_id: cnpj,
                    trade_name,
                })
            }
            PersonDataDTO::Foreign {
                passport_number,
                country_code,
            } => Ok(PersonData::Foreign {
                passport_number,
                country_code,
            }),
        }
    }
}
