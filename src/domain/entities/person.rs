use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::outbox::{AggregateType, EventType, Outbox};
use crate::domain::value_objects::{Cnpj, Cpf};

pub mod address;
pub use address::Address;

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
    pub addresses: Vec<Address>,
}

impl Person {
    pub fn new(
        id: Uuid,
        name: String,
        tenant_id: Uuid,
        created_by: Uuid,
        data: PersonData,
    ) -> Self {
        Self {
            id,
            name,
            tenant_id,
            created_by,
            created_at: Utc::now(),
            data,
            addresses: Vec::new(),
        }
    }

    pub fn add_address(&mut self, mut address: Address) {
        if address.is_main {
            for addr in &mut self.addresses {
                addr.is_main = false;
            }
        } else if self.addresses.is_empty() {
            address.is_main = true;
        }

        self.addresses.push(address);
    }

    pub fn to_outbox(&self, event_type: EventType) -> Outbox {
        let payload = serde_json::to_value(self).unwrap_or_default();
        Outbox::new(
            self.tenant_id,
            self.id,
            AggregateType::Person,
            event_type,
            payload,
            self.created_by,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::value_objects::ZipCode;

    use super::*;

    #[test]
    fn should_add_address_and_set_as_main_if_first() {
        let mut person = create_test_person();
        let address = create_test_address("Street 1", false);

        person.add_address(address);

        assert_eq!(person.addresses.len(), 1);
        assert!(person.addresses[0].is_main);
    }

    #[test]
    fn should_toggle_main_address_when_adding_new_main() {
        let mut person = create_test_person();
        let addr1 = create_test_address("Street 1", true);
        let addr2 = create_test_address("Street 2", true);

        person.add_address(addr1);
        person.add_address(addr2);

        assert_eq!(person.addresses.len(), 2);
        assert!(!person.addresses[0].is_main);
        assert!(person.addresses[1].is_main);
    }

    fn create_test_person() -> Person {
        Person::new(
            Uuid::new_v4(),
            "John Doe".to_string(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            PersonData::Individual {
                tax_id: Cpf::new("12345678909").unwrap(),
            },
        )
    }

    fn create_test_address(street: &str, is_main: bool) -> Address {
        Address::new(
            street.to_string(),
            Some("123".to_string()),
            None,
            None,
            ZipCode::new("12345678", "BR").unwrap(),
            None,
            "SP".to_string(),
            "SP".to_string(),
            "Sao Paulo".to_string(),
            "BR".to_string(),
            is_main,
            Uuid::new_v4(),
            Uuid::new_v4(),
        )
    }
}
