-- Your SQL goes here
CREATE TABLE accounts(
    id BIGSERIAL PRIMARY KEY,
    slot BIGINT NOT NULL,
    is_startup BOOLEAN NOT NULL,
    pubkey BYTEA NOT NULL,
    lamports BIGINT NOT NULL,
    owner BYTEA NOT NULL,
    executable BOOLEAN NOT NULL,
    rent_epoch BIGINT NOT NULL,
    data BYTEA NOT NULL,
    write_version BIGINT NOT NULL,
    txn_signature BYTEA
)