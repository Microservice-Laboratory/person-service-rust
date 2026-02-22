use crate::domain::entities::person::{Address, Person, PersonData};
use crate::domain::ports::output::person_repository::{PersonError, PersonRepository};
use crate::domain::value_objects::{Cnpj, Cpf, ZipCode};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use std::convert::TryFrom;
use uuid::Uuid;

pub struct PostgresPersonRepository {
    pool: PgPool,
}

impl PostgresPersonRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PersonRepository for PostgresPersonRepository {
    async fn exists_by_tax_id(&self, tax_id: &str, tenant_id: Uuid) -> Result<bool, PersonError> {
        // Verificamos nas duas tabelas de especialização
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM individual WHERE tax_id = $1 AND tenant_id = $2
                UNION ALL
                SELECT 1 FROM legal_entity WHERE business_tax_id = $1 AND tenant_id = $2
            )
            "#,
        )
        .bind(tax_id)
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| PersonError::DatabaseError(e.to_string()))?;

        Ok(exists)
    }

    async fn save(&self, person: &Person) -> Result<(), PersonError> {
        // Iniciamos uma transação para garantir o ACID e o Outbox Pattern
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| PersonError::DatabaseError(e.to_string()))?;

        // 1. Inserir na tabela base 'person'
        sqlx::query(
            r#"
            INSERT INTO person (id, type, name, tenant_id, created_by, created_at)
            VALUES ($1, $2::person_type, $3, $4, $5, $6)
            "#,
        )
        .bind(person.id)
        .bind(match person.data {
            PersonData::Individual { .. } => "individual",
            PersonData::LegalEntity { .. } => "legal_entity",
            PersonData::Foreign { .. } => "foreign",
        })
        .bind(&person.name)
        .bind(person.tenant_id)
        .bind(person.created_by)
        .bind(person.created_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| PersonError::DatabaseError(e.to_string()))?;

        // 2. Inserir na tabela de especialização
        match &person.data {
            PersonData::Individual { tax_id } => {
                sqlx::query(
                    "INSERT INTO individual (person_id, tax_id, tenant_id, created_by) VALUES ($1, $2, $3, $4)",
                )
                .bind(person.id)
                .bind(tax_id.as_str())
                .bind(person.tenant_id)
                .bind(person.created_by)
                .execute(&mut *tx)
                .await
            }
            PersonData::LegalEntity {
                business_tax_id,
                trade_name,
            } => {
                sqlx::query(
                    "INSERT INTO legal_entity (person_id, business_tax_id, trade_name, tenant_id, created_by) VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(person.id)
                .bind(business_tax_id.as_str())
                .bind(trade_name)
                .bind(person.tenant_id)
                .bind(person.created_by)
                .execute(&mut *tx)
                .await
            }
            _ => Ok(Default::default()), // Lógica para estrangeiro ou outros
        }
        .map_err(|e| PersonError::DatabaseError(e.to_string()))?;

        // 3. Inserir endereços
        for addr in &person.addresses {
            sqlx::query(
                r#"
                INSERT INTO person_addresses (
                    id, person_id, street, number, complement, neighborhood, 
                    zipcode, ibge_code, state, state_uf, city, country, 
                    is_main, tenant_id, created_by, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                "#,
            )
            .bind(addr.id)
            .bind(person.id)
            .bind(&addr.street)
            .bind(&addr.number)
            .bind(&addr.complement)
            .bind(&addr.neighborhood)
            .bind(addr.zip_code.value())
            .bind(&addr.ibge_code)
            .bind(&addr.state)
            .bind(&addr.state_uf)
            .bind(&addr.city)
            .bind(&addr.country)
            .bind(addr.is_main)
            .bind(person.tenant_id)
            .bind(addr.created_by)
            .bind(addr.created_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| PersonError::DatabaseError(e.to_string()))?;
        }

        // Finaliza a transação
        tx.commit()
            .await
            .map_err(|e| PersonError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid, tenant_id: Uuid) -> Result<Option<Person>, PersonError> {
        let row = sqlx::query(
            r#"
            SELECT 
                p.id, p.name, p.type as person_type, p.tenant_id, p.created_by, p.created_at,
                i.tax_id as individual_tax_id,
                l.business_tax_id as legal_business_tax_id,
                l.trade_name as legal_trade_name
            FROM person p
            LEFT JOIN individual i ON p.id = i.person_id
            LEFT JOIN legal_entity l ON p.id = l.person_id
            WHERE p.id = $1 AND p.tenant_id = $2
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PersonError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let person_type: String = r.get("person_type");
                let individual_tax_id: Option<String> = r.get("individual_tax_id");
                let legal_business_tax_id: Option<String> = r.get("legal_business_tax_id");
                let legal_trade_name: Option<String> = r.get("legal_trade_name");

                // Aqui acontece a "mágica" da reconstrução do polimorfismo
                let data = match person_type.as_str() {
                    "individual" => {
                        let tax_id_str = individual_tax_id.ok_or_else(|| {
                            PersonError::DatabaseError("Missing individual data".to_string())
                        })?;
                        PersonData::Individual {
                            tax_id: Cpf::try_from(tax_id_str).map_err(|e| {
                                PersonError::DatabaseError(format!(
                                    "Invalid CPF in database: {}",
                                    e
                                ))
                            })?,
                        }
                    }
                    "legal_entity" => {
                        let business_tax_id_str = legal_business_tax_id.ok_or_else(|| {
                            PersonError::DatabaseError("Missing legal entity data".to_string())
                        })?;
                        PersonData::LegalEntity {
                            business_tax_id: Cnpj::try_from(business_tax_id_str).map_err(|e| {
                                PersonError::DatabaseError(format!(
                                    "Invalid CNPJ in database: {}",
                                    e
                                ))
                            })?,
                            trade_name: legal_trade_name,
                        }
                    }
                    _ => {
                        return Err(PersonError::DatabaseError(
                            "Unknown person type".to_string(),
                        ));
                    }
                };

                // Reconstrução da lista de endereços
                let addresses_rows = sqlx::query(
                    r#"
                    SELECT 
                        id, street, number, complement, neighborhood, zipcode, 
                        ibge_code, state, state_uf, city, country, is_main, 
                        tenant_id, created_by, created_at
                    FROM person_addresses
                    WHERE person_id = $1 AND tenant_id = $2
                    "#,
                )
                .bind(id)
                .bind(tenant_id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| PersonError::DatabaseError(e.to_string()))?;

                let mut addresses = Vec::new();
                for ar in addresses_rows {
                    addresses.push(Address {
                        id: ar.get("id"),
                        street: ar.get("street"),
                        number: ar.get("number"),
                        complement: ar.get("complement"),
                        neighborhood: ar.get("neighborhood"),
                        zip_code: ZipCode::new(ar.get("zipcode"), ar.get("country"))
                            .map_err(|e| PersonError::DatabaseError(e.to_string()))?,
                        ibge_code: ar.get("ibge_code"),
                        state: ar.get("state"),
                        state_uf: ar.get("state_uf"),
                        city: ar.get("city"),
                        country: ar.get("country"),
                        is_main: ar.get("is_main"),
                        tenant_id: ar.get("tenant_id"),
                        created_by: ar.get("created_by"),
                        created_at: ar.get("created_at"),
                    });
                }

                Ok(Some(Person {
                    id: r.get("id"),
                    name: r.get("name"),
                    tenant_id: r.get("tenant_id"),
                    created_by: r.get("created_by"),
                    created_at: r.get("created_at"),
                    data,
                    addresses,
                }))
            }
            None => Ok(None),
        }
    }
}
