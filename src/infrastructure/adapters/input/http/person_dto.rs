use crate::domain::entities::person::{Address, PersonData};
use crate::domain::value_objects::{Cnpj, Cpf, ZipCode};
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
pub struct AddressDTO {
    pub street: String,
    pub number: Option<String>,
    pub complement: Option<String>,
    pub neighborhood: Option<String>,
    pub zip_code: String,
    pub ibge_code: Option<String>,
    pub state: String,
    pub state_uf: String,
    pub city: String,
    pub country: String,
    pub is_main: bool,
}

#[derive(Deserialize)]
pub struct CreatePersonRequest {
    pub name: String,
    pub data: PersonDataDTO,
    pub addresses: Option<Vec<AddressDTO>>,
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

impl AddressDTO {
    pub fn to_entity(
        self,
        tenant_id: uuid::Uuid,
        created_by: uuid::Uuid,
    ) -> Result<Address, DtoError> {
        let zip = ZipCode::new(&self.zip_code, &self.country)
            .map_err(|e| DtoError::ValidationError(e.to_string()))?;

        Ok(Address::new(
            self.street,
            self.number,
            self.complement,
            self.neighborhood,
            zip,
            self.ibge_code,
            self.state,
            self.state_uf,
            self.city,
            self.country,
            self.is_main,
            tenant_id,
            created_by,
        ))
    }
}
