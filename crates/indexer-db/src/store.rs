use anyhow::{Context, Error, Result};
use diesel::{pg::PgConnection, r2d2::{ConnectionManager, Pool}, RunQueryDsl, dsl::insert_into};

use crate::{models::NewTransaction, schema::transactions};

#[derive(Clone)]
pub struct Store {
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

impl Store {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }

    pub fn insert_transaction(&self, txs: &[NewTransaction]) -> Result<usize, Error> {
        if txs.is_empty() {
            return Ok(0);
        }

        let mut conn = self.pool.get().context("Failed to get DB connection from pool")?;

        let results = insert_into(transactions::table)
            .values(txs)
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .context("Failed to insert transactions")?;

        Ok(results)
    }
}
