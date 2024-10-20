-- Add migration script here

DROP TABLE pictures;

CREATE TABLE files(id SERIAL UNIQUE PRIMARY KEY NOT NULL, hash TEXT NOT NULL, object_storage_location TEXT NOT NULL)
