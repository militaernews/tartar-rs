use axum::{Extension, Json};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use sqlx::{PgPool, query, query_as};
use teloxide::adaptors::DefaultParseMode;
use teloxide::Bot;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{ChatId, Requester};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::error::AppError;
use crate::models::{InputReport, Report};

pub async fn redirect_readme() -> Redirect {
    Redirect::to("https://github.com/PXNX/tarta-rs#readme")
}


pub async fn user_by_id(
    Extension(db_pool): Extension<PgPool>,
    Path(user_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    query_as!(Report, r#"Select * from reports where user_id = $1 and is_banned=true"#, user_id).fetch_one(&db_pool)
        .await
        .map(Json)
        .map_err(|e|
        AppError {
            code: StatusCode::NOT_FOUND,
            message: e.to_string(),
        }
        )
}


pub async fn report_user(
    Extension(db_pool): Extension<PgPool>,
    Extension(bot): Extension<DefaultParseMode<Bot>>,
    report: Json<InputReport>,
) -> Result<impl IntoResponse, AppError> {
    let res = query_as!(Report, r#"Insert into reports (user_id, account_id, message) values ($1, $2, $3) returning *"#,
        report.user_id, 1,
        report.message)
        .fetch_one(&db_pool)
        .await?;

    send_report(&bot, db_pool).await?;

    Ok((StatusCode::CREATED, Json(res)))
}


pub async fn send_report(
    bot: &DefaultParseMode<Bot>,
    db_pool: PgPool,
) -> Result<(), AppError> {
    let reports: Vec<Report> = query_as!(Report,
        r#"select * from reports where is_banned IS NULL"#)
        .fetch_all(&db_pool)
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
                .execute(&db_pool).await?;
        }
    }

    Ok(())
}

