-- Your SQL goes here 
CREATE TABLE transactions (
    id BIGSERIAL PRIMARY KEY,
    slot BIGINT NOT NULL,
    signature BYTEA NOT NULL,
    is_vote BOOLEAN NOT NULL,
    idx INT NOT NULL,
    fee BIGINT,
    compute_units_consumed BIGINT,
    pre_balances BIGINT[] NOT NULL,
    post_balances BIGINT[] NOT NULL,
    log_messages TEXT[],
    pre_token_balances BIGINT[] NOT NULL,
    post_token_balances BIGINT[] NOT NULL
);

CREATE TABLE transaction_token_balances (
    id BIGSERIAL PRIMARY KEY,
    transaction_id BIGINT NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    account_index INT NOT NULL,
    mint TEXT NOT NULL,
    owner TEXT,
    ui_amount DOUBLE PRECISION,
    amount BIGINT,
    -- 'pre' or 'post'
    balance_type TEXT CHECK (balance_type IN ('pre', 'post'))
);

CREATE INDEX idx_transactions_signature ON transactions(signature);
CREATE INDEX idx_transactions_slot ON transactions(slot);
CREATE INDEX idx_ttb_mint ON transaction_token_balances(mint);
CREATE INDEX idx_ttb_owner ON transaction_token_balances(owner);