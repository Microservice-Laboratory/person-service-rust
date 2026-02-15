-- 1. Extensões e Tipos
-- Usamos o tipo ENUM nativo do Postgres para garantir a integridade dos tipos de pessoa
CREATE TYPE person_type AS ENUM ('individual', 'legal_entity', 'foreign');

-- 2. Tabela Principal (Base Table)
CREATE TABLE person (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    type person_type NOT NULL,
    name VARCHAR(255) NOT NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB -- Flexibilidade futura para campos não estruturados
);

-- 3. Especialização: Pessoa Física (Individual)
CREATE TABLE individual (
    person_id UUID PRIMARY KEY REFERENCES person(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL, -- Repetimos o tenant_id para facilitar o RLS e particionamento
    tax_id VARCHAR(20) NOT NULL, -- CPF
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- Unicidade do CPF dentro do mesmo Tenant (Escopo Multi-tenant)
CONSTRAINT uk_individual_tax_id_tenant UNIQUE (tax_id, tenant_id) );

-- 4. Especialização: Pessoa Jurídica (Legal Entity)
CREATE TABLE legal_entity (
    person_id UUID PRIMARY KEY REFERENCES person(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    business_tax_id VARCHAR(20) NOT NULL, -- CNPJ
    trade_name VARCHAR(255),
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- Unicidade do CNPJ dentro do mesmo Tenant
CONSTRAINT uk_legal_entity_tax_id_tenant UNIQUE (business_tax_id, tenant_id)
);

-- 6. Índices para Performance e Segurança
-- Indice para o Relay do Outbox (busca apenas eventos não processados)
-- Índices de busca por Tenant (Crucial para Shared Schema)
CREATE INDEX idx_person_tenant ON person (tenant_id);

CREATE INDEX idx_individual_tenant ON individual (tenant_id);

CREATE INDEX idx_legal_entity_tenant ON legal_entity (tenant_id);