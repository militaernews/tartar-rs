use std::env;
use std::error::Error;
use std::ops::Range;
use std::str::FromStr;

use dotenv::dotenv;
use sqlx::{PgPool, query_as};
use sqlx::postgres::PgPoolOptions;
use teloxide::{dispatching::{
    dialogue::{self, InMemStorage},
    UpdateHandler,
}, prelude::*, RequestError, types::{InlineKeyboardButton, InlineKeyboardMarkup}, utils::command::BotCommands};
use teloxide::dispatching::DefaultKey;
use teloxide::prelude::*;

use crate::dptree::di::DependencySupplier;
use crate::models::User;

mod models;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let bot: AutoSend<Bot> = Bot::from_env().auto_send();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&*env::var("DATABASE_URL").expect("DATABASE_URL must be provided!")).await.unwrap();


    let handler = dptree::entry()
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![pool])


        .build().setup_ctrlc_handler().dispatch().await
}


async fn callback_handler(
    q: CallbackQuery,
    bot: AutoSend<Bot>,
    db_pool: PgPool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("click");

    let result: User = query_as!(User, r#"insert into users values($1, $2, current_date) returning *"#, 1234, String::from("message")).fetch_one(&db_pool).await?;

    if let Some(report_id) = q.data {
        match q.message {
            Some(Message { id, chat, .. }) => {
                bot.edit_message_reply_markup(chat.id, id).await?;
            }

            _ => {}
        }
    }

    Ok(()) //     respond(())
}