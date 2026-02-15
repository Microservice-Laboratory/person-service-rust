use crate::domain::entities::person::{Person, PersonData};
use crate::domain::ports::output::person_repository::{PersonError, PersonRepository};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
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
                .bind(tax_id)
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
                .bind(business_tax_id)
                .bind(trade_name)
                .bind(person.tenant_id)
                .bind(person.created_by)
                .execute(&mut *tx)
                .await
            }
            _ => Ok(Default::default()), // Lógica para estrangeiro ou outros
        }
        .map_err(|e| PersonError::DatabaseError(e.to_string()))?;

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
                    "individual" => PersonData::Individual {
                        tax_id: individual_tax_id.ok_or_else(|| {
                            PersonError::DatabaseError("Missing individual data".to_string())
                        })?,
                    },
                    "legal_entity" => PersonData::LegalEntity {
                        business_tax_id: legal_business_tax_id.ok_or_else(|| {
                            PersonError::DatabaseError("Missing legal entity data".to_string())
                        })?,
                        trade_name: legal_trade_name,
                    },
                    _ => {
                        return Err(PersonError::DatabaseError(
                            "Unknown person type".to_string(),
                        ));
                    }
                };

                Ok(Some(Person {
                    id: r.get("id"),
                    name: r.get("name"),
                    tenant_id: r.get("tenant_id"),
                    created_by: r.get("created_by"),
                    created_at: r.get("created_at"),
                    data,
                }))
            }
            None => Ok(None),
        }
    }
}
