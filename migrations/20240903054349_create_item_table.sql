-- Add migration script here

CREATE TABLE items(id SERIAL UNIQUE PRIMARY KEY NOT NULL, name TEXT NOT NULL, desciption TEXT NOT NULL, date_origin TIMESTAMP WITH TIME ZONE NOT NULL, date_recieved TIMESTAMP WITH TIME ZONE NOT NULL)
