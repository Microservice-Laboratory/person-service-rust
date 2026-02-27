-- Add migration script here
CREATE TABLE outboxes (
    id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    aggregate_id UUID NOT NULL,
    aggregate_type VARCHAR(100) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    processed_at TIMESTAMP WITH TIME ZONE,
    PRIMARY KEY (id, tenant_id)
);

CREATE INDEX idx_outboxes_tenant_aggregate ON outboxes (tenant_id, aggregate_id);
CREATE INDEX idx_outboxes_unprocessed ON outboxes (tenant_id) WHERE processed_at IS NULL;
