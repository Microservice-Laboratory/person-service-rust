-- Add migration script here
CREATE TABLE person_addresses (
    id UUID NOT NULL,
    person_id UUID NOT NULL,
    street VARCHAR(255) NOT NULL,
    number VARCHAR(20),
    complement VARCHAR(100),
    neighborhood VARCHAR(100),
    zipcode VARCHAR(8),
    ibge_code VARCHAR(7),
    state VARCHAR(100),
    state_uf VARCHAR(2),
    city VARCHAR(255),
    country VARCHAR(100) NOT NULL,
    is_main BOOLEAN DEFAULT FALSE,
    tenant_id UUID NOT NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_person_addresses PRIMARY KEY (id, tenant_id),
    CONSTRAINT fk_person_addresses_person FOREIGN KEY (person_id) REFERENCES person(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX idx_one_main_address ON person_addresses (person_id, tenant_id) WHERE is_main = true;
