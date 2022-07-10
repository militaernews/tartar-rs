use std::error::Error;
use std::{convert::Infallible, env, net::SocketAddr};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::Filter;
use reqwest::{StatusCode, Url};

use std::time::Duration;

use dotenv::dotenv;
use sqlx::{Pool, Postgres, query, query_as};
use sqlx::postgres::PgPoolOptions;
use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup}, utils::command::BotCommands};



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
                    let _result = query!(
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

            query!(
        r#"update reports set is_banned = false where id = $1"#,
        &report.id)
                .execute(pool)
                .await?;
        }
    }


    Ok(())
}

use teloxide::{
    dispatching::{
        stop_token::AsyncStopToken,
        update_listeners::{self, StatefulListener},
    },
    prelude::*,
    types::Update,
};


async fn handle_rejection(error: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    log::error!("Cannot process the request due to: {:?}", error);
    Ok(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn webhook(bot: AutoSend<Bot>) -> impl update_listeners::UpdateListener<Infallible> {
    // Heroku auto defines a port value
    let teloxide_token = env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN env variable missing");
    let port: u16 = env::var("PORT")
        .expect("PORT env variable missing")
        .parse()
        .expect("PORT value to be integer");
    // Heroku host example .: "heroku-ping-pong-bot.herokuapp.com"
    let host = env::var("HOST").expect("have HOST env variable");
    let path = format!("bot{teloxide_token}");
    let url = Url::parse(&format!("https://{host}/{path}")).unwrap();

    bot.set_webhook(url).await.expect("Cannot setup a webhook");

    let (tx, rx) = mpsc::unbounded_channel();

    let server = warp::post()
        .and(warp::path(path))
        .and(warp::body::json())
        .map(move |update: Update| {
            tx.send(Ok(update)).expect("Cannot send an incoming update from the webhook");

            StatusCode::OK
        })
        .recover(handle_rejection);

    let (stop_token, stop_flag) = AsyncStopToken::new_pair();

    let addr = format!("0.0.0.0:{port}").parse::<SocketAddr>().unwrap();
    let server = warp::serve(server);
    let (_addr, fut) = server.bind_with_graceful_shutdown(addr, stop_flag);

    // You might want to use serve.key_path/serve.cert_path methods here to
    // setup a self-signed TLS certificate.

    tokio::spawn(fut);
    let stream = UnboundedReceiverStream::new(rx);

    fn streamf<S, T>(state: &mut (S, T)) -> &mut S {
        &mut state.0
    }

    StatefulListener::new((stream, stop_token), streamf, |state: &mut (_, AsyncStopToken)| {
        state.1.clone()
    })
}