use std::env;

use deadpool_postgres::Client;
use sqlx::{PgConnection, PgPool, Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use tokio_pg_mapper::FromTokioPostgresRow;
use crate::error::MyError;

use crate::model::{InputUser, User};



pub async fn get_pool() -> PgPool {

    let db_url = env::var("DATABASE_URL").unwrap();

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&*String::from(db_url))
        .await.unwrap()
}


pub async fn add_user(conn: &PgPool, user: InputUser) -> Result<User, sqlx::Error>{ //Result<User, MyError> {
sqlx::query_as!(User,
        r#"INSERT INTO users(id, msg, date) VALUES ($1, $2, current_date) RETURNING id, msg, date;"#, user.id, user.msg
    )
        .fetch_one(&*conn)
        .await?




 /*       .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect::<Vec<User>>()
        .pop()
        .ok_or(MyError::NotFound) // more applicable for SELECTs

  */
}
