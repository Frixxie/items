--liquibase formatted sql

--changeset frixxie:1
CREATE SCHEMA prestgaarsbasen;
CREATE TABLE prestgaarsbasen.gifter (
    id SERIAL UNIQUE,
    firstname text,
    lastname text
);
CREATE TABLE prestgaarsbasen.items (
    id SERIAL UNIQUE,
    name text,
    gifterid int references prestgaarsbasen.gifter(id),
    pictureurl text,
    description text
);
CREATE TABLE prestgaarsbasen.tags (
    id SERIAL UNIQUE references prestgaarsbasen.items(id),
    tag1 bool DEFAULT false
);
--rollback DROP TABLE prestgaarsbasen.gifter;
--rollback DROP TABLE prestgaarsbasen.items;
--rollback DROP TABLE prestgaarsbasen.tags;

