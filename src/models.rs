use chrono::{NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Report {
    pub id: i32,
    pub message: String,
    pub user_id: i32,
    pub account_id: i32,
    pub reported_at: NaiveDateTime,
    pub is_banned: Option<bool>,

}