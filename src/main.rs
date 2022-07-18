use std::{env, error::Error, sync::Arc, time::Duration};

use dotenv::dotenv;
use reqwest::Url;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions, query, query_as};
use teloxide::{dispatching::update_listeners::webhooks, error_handlers::IgnoringErrorHandlerSafe,
               prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup, Update}};
use tokio::time::sleep;

use crate::models::Report;

mod models;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting tartaros-telegram...");

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
            send_report(&b, &p).await.unwrap();
            sleep(Duration::from_millis(5000)).await;
        }
    });

    let token = bot.inner().token();

    // Heroku auto defines a port value
    let port: u16 = env::var("PORT")
        .expect("PORT env variable is not set")
        .parse()
        .expect("PORT env variable value is not an integer");

    let addr = ([0, 0, 0, 0], port).into();

    // Heroku host example: "heroku-ping-pong-bot.herokuapp.com"
    let host = env::var("HOST").expect("HOST env variable is not set");
    let url = Url::parse(&format!("https://{host}/webhooks/{token}")).unwrap();

    let listener = webhooks::axum(bot.clone(), webhooks::Options::new(addr, url))
        .await
        .expect("Couldn't setup webhook");

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![pool])
        .build()
        .setup_ctrlc_handler()
        .dispatch_with_listener(listener, Arc::new(IgnoringErrorHandlerSafe))
        .await
}


async fn callback_handler(
    q: CallbackQuery,
    bot: AutoSend<Bot>,
    pool: Pool<Postgres>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("click");

    if let Some(report) = q.data {
        if let Some(Message { id, chat, .. }) = q.message {
            let ban: char = report.chars().next().unwrap();
            let report_id: i32 = report[1..report.len()].parse::<i32>().unwrap();

            println!("ban: {} - id: {}", ban, report_id);

            if ban == 'y' {
                let _result = query!(r#"update reports set is_banned = true where id = $1"#,report_id)
                    .execute(&pool).await?;
            }

            //maybe edit text and append "reported" or "declined" ?
            bot.edit_message_reply_markup(chat.id, id).await?;
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

            query!(r#"update reports set is_banned = false where id = $1"#,&report.id)
                .execute(pool).await?;
        }
    }

    Ok(())
}