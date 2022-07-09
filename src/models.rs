use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct User {
    pub id: i32,
    pub msg: String,
    pub date: NaiveDateTime,
}