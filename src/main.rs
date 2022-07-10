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
use teloxide::types::{Chat, ParseMode};
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

    tokio::spawn(async move {
        loop {
            println!("hm");
            send_report(&b, &p).await;
            sleep(Duration::from_millis(5000)).await;
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


    if let Some(report) = q.data {
        match q.message {
            Some(Message { id, chat, .. }) => {
                let ban: char = report.chars().next().unwrap();
                let report_id: i32 = report[1..report.len()].parse::<i32>().unwrap();

                println!("ban: {} - id: {}", ban, report_id);

                if ban == 'y' {
                    let result = query!(
        r#"update reports set is_banned = true where id = $1"#,
       report_id)
                        .execute(&pool)
                        .await?;
                }


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
        .fetch_all(*&pool)
        .await?;

    println!("Report length {}", reports.len());

    if !reports.is_empty() {
        for report in reports {
            println!("{}", &report.id);

            let keyboard = InlineKeyboardMarkup::new(vec![
                vec![
                    InlineKeyboardButton::callback("Ban user ✅️", format!("y{}", &report.id))
                ],
                vec![
                    InlineKeyboardButton::callback("Cancel report 🚫", format!("n{}", &report.id))
                ]]);

            //TODO: .parse_mode(ParseMode::Html) and format user_id to link to user
            bot.send_message(ChatId(-1001758396624),
                             format!("📋 {} - ✍️ {} - 🧑 {}\n\n{}",
                                     &report.id, &report.account_id, &report.user_id, &report.message))
                .reply_markup(keyboard).await.expect("Failed to send message");

            query!(
        r#"update reports set is_banned = false where id = $1"#,
        &report.id)
                .execute(pool)
                .await?;
        }
    }


    Ok(())
}