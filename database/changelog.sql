--liquibase formatted sql

--changeset frixxie:1
CREATE SCHEMA prestgaarsbasen;
CREATE TABLE prestgaarsbasen.gifter (
    id SERIAL UNIQUE,
    firstname text,
    lastname text
);
CREATE TABLE prestgaarsbasen.items (
    id SERIAL PRIMARY KEY,
    name text,
    gifterid int references prestgaarsbasen.gifter(id),
    pictureurl text,
    description text
);
CREATE TABLE prestgaarsbasen.tags (
    id SERIAL PRIMARY KEY,
    name text,
);
CREATE TABLE prestgaarsbasen.itemtags (
    itemid SERIAL references prestgaarsbasen.items(id),
    tagid SERIAL references prestgaarsbasen.tags(id),
    created timestamp,
    PRIMARY KEY (itemid, tagid)
)
--rollback DROP TABLE prestgaarsbasen.gifter;
--rollback DROP TABLE prestgaarsbasen.items;
--rollback DROP TABLE prestgaarsbasen.tags;

