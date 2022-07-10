use std::env;
use std::error::Error;
use std::ops::Range;
use std::str::FromStr;
use std::time::Duration;

use dotenv::dotenv;
use sqlx::{PgPool, Pool, Postgres, query, query_as};
use sqlx::postgres::PgPoolOptions;
use teloxide::{dispatching::{
    dialogue::{self, InMemStorage},
    UpdateHandler,
}, prelude::*, RequestError, types::{InlineKeyboardButton, InlineKeyboardMarkup}, utils::command::BotCommands};
use teloxide::dispatching::DefaultKey;
use teloxide::prelude::*;
use tokio::time::sleep;

use crate::dptree::di::DependencySupplier;
use crate::models::{Report};

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


    let b = bot.clone();
    let p = pool.clone();

    tokio::spawn( async move {


        loop {
            println!("hm");
            send_report(&b, &p).await;
            sleep(Duration::from_millis(3000)).await;
        }
    });


    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![pool])
        .build().setup_ctrlc_handler().dispatch().await
}


async fn callback_handler(
    q: CallbackQuery,
    bot: AutoSend<Bot>,
    pool: Pool<Postgres>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("click");

    let result= query!(
        r#"update reports set is_banned = true where id = $1"#,
        1234)
        .execute(&pool)
        .await?;

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

async fn send_report(
    bot: &AutoSend<Bot>,
    pool: &Pool<Postgres>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let reports: Vec<Report> = query_as!(Report,
        r#"select * from reports where is_banned IS NULL"#)
        .fetch_all(pool)
        .await?;

    println!("{}", reports.len());

    for report in reports{
        println!("{}",report.id)
    }

    Ok(())
}