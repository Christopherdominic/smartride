-- Separate public demo data from existing merchant records.
ALTER TABLE merchants ADD COLUMN is_demo BOOLEAN NOT NULL DEFAULT FALSE;
CREATE INDEX idx_delivery_events_timeline ON delivery_events(delivery_id, created_at);
ALTER TABLE delivery_otps ADD CONSTRAINT otp_attempts CHECK (attempts BETWEEN 0 AND 5);
