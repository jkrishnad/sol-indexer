use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
    pub id: i64,
    pub slot: i64,
    pub signature: Vec<u8>,
    pub is_vote: bool,
    pub idx: i32,
    pub fee: Option<i64>,
    pub compute_units_consumed: Option<i64>,
    pub pre_balances: Vec<Option<i64>>,
    pub post_balances: Vec<Option<i64>>,
    pub log_messages: Option<Vec<Option<String>>>,
    pub pre_token_balances: Vec<Option<i64>>,
    pub post_token_balances: Vec<Option<i64>>,
}

#[derive(Insertable, Debug,Deserialize,Serialize)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransaction {
    pub slot: i64,
    pub signature: Vec<u8>,
    pub is_vote: bool,
    pub idx: i32,
    pub fee: Option<i64>,
    pub compute_units_consumed: Option<i64>,
    pub pre_balances: Vec<Option<i64>>,
    pub post_balances: Vec<Option<i64>>,
    pub log_messages: Option<Vec<Option<String>>>,
    pub pre_token_balances: Vec<Option<i64>>,
    pub post_token_balances: Vec<Option<i64>>,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::transaction_token_balances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionTokenBAlance {
    pub id: i64,
    pub transaction_id: i64,
    pub account_index: i32,
    pub mint: String,
    pub owner: Option<String>,
    pub ui_amount: Option<f64>,
    pub amount: Option<i64>,
    pub balance_type: Option<String>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::transaction_token_balances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransactionTokenBalance {
    pub transaction_id: i64,
    pub account_index: i32,
    pub mint: String,
    pub owner: Option<String>,
    pub ui_amount: Option<f64>,
    pub amount: Option<i64>,
    pub balance_type: Option<String>,
}

impl NewTransaction {
    pub fn new(
        slot: i64,
        signature: Vec<u8>,
        is_vote: bool,
        idx: i32,
        fee: Option<i64>,
        compute_units_consumed: Option<i64>,
        pre_balances: Option<Vec<Option<i64>>>,
        post_balances: Option<Vec<Option<i64>>>,
        log_messages: Option<Vec<Option<String>>>,
        pre_token_balances: Option<Vec<Option<i64>>>,
        post_token_balances: Option<Vec<Option<i64>>>,
    ) -> Self {
        NewTransaction {
            slot,
            signature,
            is_vote,
            idx,
            fee,
            compute_units_consumed,
            pre_balances: pre_balances.unwrap_or_default(),
            post_balances: post_balances.unwrap_or_default(),
            log_messages,
            pre_token_balances: pre_token_balances.unwrap_or_default(),
            post_token_balances: post_token_balances.unwrap_or_default(),
        }
    }
}
