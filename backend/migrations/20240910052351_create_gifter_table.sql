-- Add migration script here

CREATE TABLE gifters(id SERIAL UNIQUE PRIMARY KEY NOT NULL, firstname TEXT NOT NULL, lastname TEXT NOT NULL, notes TEXT NOT NULL, date_added TIMESTAMP WITH TIME ZONE NOT NULL)
