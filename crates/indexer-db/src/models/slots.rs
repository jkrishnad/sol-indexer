use crate::schema::slots;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = slots)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Slot {
    pub id: i32,
    pub slot: i64,
    pub parent: Option<i64>,
    pub status: i32,
    pub dead_error: Option<String>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = slots)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewSlot {
    pub slot: i64,
    pub parent: Option<i64>,
    pub status: i32,
    pub dead_error: Option<String>,
}

impl Slot {
    pub fn new(slot: i64, parent: Option<i64>, status: i32, dead_error: Option<String>) -> NewSlot {
        NewSlot {
            slot,
            parent,
            status,
            dead_error,
        }
    }
}
