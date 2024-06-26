use std::error::Error;
use sqlx::{Pool, Postgres, query};
use teloxide::utils::command::BotCommands;
use teloxide_core::adaptors::DefaultParseMode;
use teloxide_core::Bot;
use teloxide_core::prelude::Requester;
use teloxide_core::requests::ResponseResult;
use teloxide_core::types::{CallbackQuery, Message};



#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "Start the bot")]
    Start,

}

pub async fn commands(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Start => bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,

    };

    Ok(())
}

pub async fn callback_handler(
    q: CallbackQuery,
    bot: DefaultParseMode<Bot>,
    pool: Pool<Postgres>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("click");

    if let (Some(report), Some(message)) = (q.data, q.message) {
        let ban: char = report.chars().next().unwrap();
        let report_id: i32 = report[1..report.len()].parse::<i32>().unwrap();

        println!("ban: {} - id: {}", ban, report_id);

        match ban {
            'y' => {
                let _result = query!(r#"update reports set is_banned = true where id = $1"#,report_id)
                    .execute(&pool).await?;
                bot.edit_message_text(message.chat.id, message.id, format!("{}\n\nUser banned ✅️", message.text().unwrap())).await?;
            }
            'n' => {
                bot.edit_message_text(message.chat.id, message.id, format!("{}\n\nReport cancelled 🚫️", message.text().unwrap())).await?;
            }
            _ => {}
        }

        bot.edit_message_reply_markup(message.chat.id, message.id).await?;
    }

    Ok(())
}

