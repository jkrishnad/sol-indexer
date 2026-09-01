use crate::schema::accounts;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = accounts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Account {
    pub id: i64,
    pub slot: i64,
    pub is_startup: bool,
    pub pubkey: Vec<u8>,
    pub lamports: i64,
    pub owner: Vec<u8>,
    pub executable: bool,
    pub rent_epoch: i64,
    pub data: Vec<u8>,
    pub write_version: i64,
    pub txn_signature: Option<Vec<u8>>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = accounts)]
pub struct NewAccount {
    pub slot: i64,
    pub is_startup: bool,
    pub pubkey: Vec<u8>,
    pub lamports: i64,
    pub owner: Vec<u8>,
    pub executable: bool,
    pub rent_epoch: i64,
    pub data: Vec<u8>,
    pub write_version: i64,
    pub txn_signature: Option<Vec<u8>>,
}

impl NewAccount {
    pub fn new(
        slot: i64,
        is_startup: bool,
        pubkey: Vec<u8>,
        lamports: i64,
        owner: Vec<u8>,
        executable: bool,
        rent_epoch: i64,
        data: Vec<u8>,
        write_version: i64,
        txn_signature: Option<Vec<u8>>,
    ) -> Self {
        NewAccount {
            slot,
            is_startup,
            pubkey,
            lamports,
            owner,
            executable,
            rent_epoch,
            data,
            write_version,
            txn_signature,
        }
    }
}
