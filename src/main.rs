use std::{env, error::Error, sync::Arc};
use std::future::IntoFuture;
use std::net::SocketAddr;

use anyhow::anyhow;
use axum::{Extension, Router};
use axum::extract::FromRef;
use axum::routing::{get, post};
use dotenv::dotenv;
use reqwest::Url;
use sqlx::{PgPool, Pool, Postgres, postgres::PgPoolOptions, query, query_as};
use teloxide::{error_handlers::IgnoringErrorHandlerSafe, filter_command, prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup, Update}};
use teloxide::adaptors::DefaultParseMode;
use teloxide::types::ParseMode;
use teloxide::update_listeners::webhooks::{axum_to_router, Options};
use tokio::net::TcpListener;

use crate::bot::{callback_handler, Command, commands};
use crate::models::{InputReport, Report};
use crate::routes::{redirect_readme, report_user, user_by_id};

mod models;
mod routes;
mod error;
mod bot;


#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting tartar-rs...");

    let port: u16 = env::var("PORT")
        .expect("PORT env variable is not set")
        .parse()
        .expect("PORT env variable value is not an integer");
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let bot: DefaultParseMode<Bot> = Bot::from_env().parse_mode(ParseMode::Html);

    let token = bot.inner().token();
    let host = env::var("HOST").expect("HOST env variable is not set");
    let url = Url::parse(&format!("https://{host}/webhooks/{token}")).unwrap();

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&*env::var("DATABASE_URL").expect("DATABASE_URL must be provided!")).await.unwrap();
    let b = bot.clone();

    let (mut update_listener, stop_flag, app) = axum_to_router(bot.clone(), Options::new(addr, url)).await
        .expect("Couldn't setup webhook");


    // let command_handler = Command::repl(&bot, commands).await.into();

    let handler = dptree::entry()

        .branch(Update::filter_callback_query().endpoint(callback_handler))
        //     .branch(command_handler)
        ;
    let mut dp = Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![db_pool.clone()])
        .build();
    let d = dp
        .dispatch_with_listener(update_listener, Arc::new(IgnoringErrorHandlerSafe));

    let app = app
        .route("/", get(redirect_readme))
        .route("/reports", post(report_user))
        .route("/users/:user_id", get(user_by_id))
        .layer(Extension(b))
        .layer(Extension(db_pool));

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");
    let server = axum::serve(listener, app).into_future()
        ;
    //    .expect("Server error.");

    let (_, _) = tokio::join!(server, d);
}



