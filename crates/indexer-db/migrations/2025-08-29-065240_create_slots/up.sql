-- Your SQL goes here
CREATE TABLE slots (
    id SERIAL PRIMARY KEY,
    slot BIGINT NOT NULL,
    parent BIGINT,
    status INTEGER NOT NULL,
    dead_error TEXT
);