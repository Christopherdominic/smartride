-- L1 verifies postcode validity only. Customer-supplied addresses have separate provenance.
-- Existing offline demo addresses and delivery histories remain untouched.
ALTER TABLE addresses ADD COLUMN customer_address TEXT;
ALTER TABLE addresses ADD COLUMN landmark TEXT;
ALTER TABLE addresses ADD COLUMN postcode_verified_at TIMESTAMPTZ;
ALTER TABLE addresses ADD CONSTRAINT customer_address_length CHECK (customer_address IS NULL OR char_length(customer_address) BETWEEN 5 AND 1000);
ALTER TABLE addresses ADD CONSTRAINT landmark_length CHECK (landmark IS NULL OR char_length(landmark) <= 500);
