-- Add migration script here

DROP TABLE gifters CASCADE;

ALTER TABLE items DROP COLUMN date_recieved
