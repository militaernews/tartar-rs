use std::{env, error::Error, sync::Arc};
use std::future::IntoFuture;
use std::net::SocketAddr;

use anyhow::anyhow;
use axum::{Extension, Router, serve};
use axum::routing::{get, post};
use dotenv::dotenv;
use reqwest::Url;
use sqlx::{PgPool, postgres::PgPoolOptions};
use teloxide::{error_handlers::IgnoringErrorHandlerSafe, prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup, Update}};
use teloxide::adaptors::DefaultParseMode;
use teloxide::dptree::deps;
use teloxide::types::ParseMode;
use teloxide::update_listeners::webhooks::{axum_to_router, Options};
use teloxide_core::types::ParseMode::Html;
use tokio::join;
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

    let bot: DefaultParseMode<Bot> = Bot::from_env().parse_mode(Html);

    let token = bot.inner().token();
    let host = env::var("HOST").expect("HOST env variable is not set");
    let url = Url::parse(&format!("https://{host}/webhooks/{token}")).unwrap();

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&*env::var("DATABASE_URL").expect("DATABASE_URL must be provided!")).await.unwrap();

    let (update_listener, stop_flag, app) = axum_to_router(bot.clone(), Options::new(addr, url)).await
        .expect("Couldn't setup webhook");

    let handler = dptree::entry()
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    let mut dp = Dispatcher::builder(bot.clone(), handler)
        .dependencies(deps![db_pool.clone()])
        .build();
    let d = dp
        .dispatch_with_listener(update_listener, Arc::new(IgnoringErrorHandlerSafe));

    let app = app
        .route("/", get(redirect_readme))
        .route("/reports", post(report_user))
        .route("/users/:user_id", get(user_by_id))
        .layer(Extension(bot))
        .layer(Extension(db_pool));

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");
    let server = serve(listener, app).into_future();

    let (_, _) = join!(server, d);
}
