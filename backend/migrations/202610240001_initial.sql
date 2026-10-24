CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE merchants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE riders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    name TEXT NOT NULL,
    phone TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE addresses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    postcode TEXT NOT NULL,
    formatted_address TEXT,
    provider TEXT NOT NULL,
    provider_data JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    rider_id UUID REFERENCES riders(id),
    address_id UUID NOT NULL REFERENCES addresses(id),
    external_reference TEXT,
    recipient_name TEXT NOT NULL,
    recipient_phone TEXT NOT NULL,
    parcel_description TEXT,
    delivery_instructions TEXT,
    status TEXT NOT NULL DEFAULT 'created'
        CHECK (status IN (
            'created',
            'assigned',
            'picked_up',
            'out_for_delivery',
            'delivered',
            'failed',
            'cancelled'
        )),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (merchant_id, external_reference)
);

CREATE INDEX idx_deliveries_merchant
ON deliveries(merchant_id, created_at DESC);

CREATE INDEX idx_deliveries_rider
ON deliveries(rider_id);

CREATE TABLE delivery_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    delivery_id UUID NOT NULL REFERENCES deliveries(id),
    event_type TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE delivery_otps (
    delivery_id UUID PRIMARY KEY REFERENCES deliveries(id),
    code_hash TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    consumed_at TIMESTAMPTZ
);
