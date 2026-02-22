use crate::domain::entities::person::{Address, Person, PersonData};
use crate::domain::ports::output::person_repository::{PersonError, PersonRepository};
use uuid::Uuid;

// O que o mundo externo envia para o caso de uso
pub struct CreatePersonCommand {
    pub tenant_id: Uuid,
    pub name: String,
    pub created_by: Uuid,
    pub data: PersonData,
    pub addresses: Vec<Address>,
}

pub struct CreatePersonUseCase<R: PersonRepository> {
    repository: R,
}

impl<R: PersonRepository> CreatePersonUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, cmd: CreatePersonCommand) -> Result<Person, PersonError> {
        // 1. Lógica de Negócio: Verificar se já existe alguém com esse documento no mesmo tenant
        let tax_id = match &cmd.data {
            PersonData::Individual { tax_id } => tax_id.as_str(),
            PersonData::LegalEntity {
                business_tax_id, ..
            } => business_tax_id.as_str(),
            PersonData::Foreign {
                passport_number, ..
            } => passport_number.as_str(),
        };

        if self
            .repository
            .exists_by_tax_id(tax_id, cmd.tenant_id)
            .await?
        {
            return Err(PersonError::Conflict(format!(
                "Tax ID {} already exists for this tenant",
                tax_id
            )));
        }

        // 2. Criação da Entidade de Domínio (Onde as regras de negócio residem)
        let mut person = Person::new(
            Uuid::new_v4(), // No Rust, geramos o UUID no domínio para manter a posse do ID
            cmd.name,
            cmd.tenant_id,
            cmd.created_by,
            cmd.data,
        );

        for addr in cmd.addresses {
            person.add_address(addr);
        }

        // 3. Persistência através do Port de Saída (Adapter será chamado aqui)
        self.repository.save(&person).await?;

        Ok(person)
    }
}
