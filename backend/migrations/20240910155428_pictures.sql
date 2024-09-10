-- Add migration script here

CREATE TABLE pictures(id SERIAL UNIQUE PRIMARY KEY NOT NULL, item_id INTEGER NOT NULL REFERENCES items (id), description TEXT NOT NULL, hash TEXT NOT NULL, object_storage_location TEXT NOT NULL)
