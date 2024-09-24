-- Add migration script here

CREATE TABLE locations(id SERIAL UNIQUE PRIMARY KEY NOT NULL, name TEXT NOT NULL, description TEXT NOT NULL)
