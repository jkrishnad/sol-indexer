// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (id) {
        id -> Int8,
        slot -> Int8,
        is_startup -> Bool,
        pubkey -> Bytea,
        lamports -> Int8,
        owner -> Bytea,
        executable -> Bool,
        rent_epoch -> Int8,
        data -> Bytea,
        write_version -> Int8,
        txn_signature -> Nullable<Bytea>,
    }
}

diesel::table! {
    slots (id) {
        id -> Int4,
        slot -> Int8,
        parent -> Nullable<Int8>,
        status -> Int4,
        dead_error -> Nullable<Text>,
    }
}

diesel::table! {
    transaction_token_balances (id) {
        id -> Int8,
        transaction_id -> Int8,
        account_index -> Int4,
        mint -> Text,
        owner -> Nullable<Text>,
        ui_amount -> Nullable<Float8>,
        amount -> Nullable<Int8>,
        balance_type -> Nullable<Text>,
    }
}

diesel::table! {
    transactions (id) {
        id -> Int8,
        slot -> Int8,
        signature -> Bytea,
        is_vote -> Bool,
        idx -> Int4,
        fee -> Nullable<Int8>,
        compute_units_consumed -> Nullable<Int8>,
        pre_balances -> Array<Nullable<Int8>>,
        post_balances -> Array<Nullable<Int8>>,
        log_messages -> Nullable<Array<Nullable<Text>>>,
        pre_token_balances -> Array<Nullable<Int8>>,
        post_token_balances -> Array<Nullable<Int8>>,
    }
}

diesel::joinable!(transaction_token_balances -> transactions (transaction_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    slots,
    transaction_token_balances,
    transactions,
);
