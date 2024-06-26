use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Report {
    pub id: i32,
    pub message: String,
    pub user_id: i64,
    pub account_id: i32,
    pub reported_at: DateTime<Utc>,
    pub is_banned: Option<bool>,
}

pub struct Account {
    pub id: i32,
    pub api_key: String,
    pub valid_until: DateTime<Utc>,
}

pub struct NewReport {
    pub message: String,
    pub user_id: i64,
    pub account_id: i32,
}

#[derive(Deserialize, Debug)]
pub struct InputReport {
    pub message: String,
    pub user_id: i64,
}

pub struct User {
    pub id: i64,
    pub banned_since: DateTime<Utc>,
    pub messages: Vec<String>,
}

