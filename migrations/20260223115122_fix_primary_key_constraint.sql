-- Fix primary key and foreign key constraints for multi-tenancy

-- 1. Remove existing foreign keys that reference person(id)
-- These need to be removed before we can change the primary key of the person table
ALTER TABLE individual DROP CONSTRAINT IF EXISTS individual_person_id_fkey;
ALTER TABLE legal_entity DROP CONSTRAINT IF EXISTS legal_entity_person_id_fkey;
ALTER TABLE person_addresses DROP CONSTRAINT IF EXISTS fk_person_addresses_person;

-- 2. Remove existing primary keys
ALTER TABLE person DROP CONSTRAINT IF EXISTS person_pkey;
ALTER TABLE individual DROP CONSTRAINT IF EXISTS individual_pkey;
ALTER TABLE legal_entity DROP CONSTRAINT IF EXISTS legal_entity_pkey;

-- 3. Add new composite primary keys (id/person_id, tenant_id)
ALTER TABLE person ADD PRIMARY KEY (id, tenant_id);
ALTER TABLE individual ADD PRIMARY KEY (person_id, tenant_id);
ALTER TABLE legal_entity ADD PRIMARY KEY (person_id, tenant_id);

-- 4. Re-add foreign keys referencing the new composite primary key of person
-- This ensures that specialized entities and addresses are correctly linked to a person within the same tenant
ALTER TABLE individual
    ADD CONSTRAINT fk_individual_person
    FOREIGN KEY (person_id, tenant_id)
    REFERENCES person (id, tenant_id)
    ON DELETE CASCADE;

ALTER TABLE legal_entity
    ADD CONSTRAINT fk_legal_entity_person
    FOREIGN KEY (person_id, tenant_id)
    REFERENCES person (id, tenant_id)
    ON DELETE CASCADE;

ALTER TABLE person_addresses
    ADD CONSTRAINT fk_person_addresses_person
    FOREIGN KEY (person_id, tenant_id)
    REFERENCES person (id, tenant_id)
    ON DELETE CASCADE;
