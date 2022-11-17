use miette::{bail, Context as _, IntoDiagnostic, Result};
use rand::seq::SliceRandom;
use std::env;
use teloxide::{
    dispatching::UpdateHandler,
    prelude::*,
    types::{MediaKind, MessageKind, ParseMode},
    utils::command::BotCommands,
};
use tracing::{debug, info, warn};

use crate::{
    common::{bot::respond, text},
    db::{models::*, sqlite::*},
    utterance::DialogflowSession,
};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "view L scoreboard")]
    ViewScoreboard,
    #[command(description = "award L to user")]
    GiveL,
    #[command(description = "learn a new phrase")]
    Learn(String),
}

impl Command {
    async fn from_phrase(input: String, me: teloxide::types::Me) -> Option<Self> {
        let response = DialogflowSession::new()
            .await
            .ok()?
            .detect_intent_from_text(input, None)
            .await
            .ok()?;

        debug!("Dialogflow response: {:#?}", response);

        if let Some(result) = &response.get_ref().query_result {
            let mcmd = Self::bot_commands()
                .iter()
                .find(|cmd| cmd.command == result.action)?
                .to_owned();

            Self::parse(&mcmd.command, me.username()).ok()
        } else {
            None
        }
    }

    async fn handle(
        &self,
        bot: Bot,
        me: teloxide::types::Me,
        msg: Message,
    ) -> Result<Option<String>> {
        let Some(author) = msg.from() else {
            bail!("Message has no author");
        };

        match self {
            Self::Help => {
                bot.send_message(msg.chat.id, {
                    if msg.chat.is_group() || msg.chat.is_supergroup() {
                        Command::descriptions().username_from_me(&me).to_string()
                    } else {
                        Command::descriptions().to_string()
                    }
                })
                .await
                .into_diagnostic()?;
            }
            Self::ViewScoreboard => {
                let stats = sqlx::query_as::<_, MemberStat>(
                    r#"
                    SELECT tgUserId, ls
                    FROM Member
                    LEFT JOIN Stat
                           ON Member.id = Stat.memberId
                    "#,
                )
                .fetch_all(db())
                .await
                .into_diagnostic()?;

                if stats.is_empty() {
                    respond!("Scoreboard is empty\\!");
                }

                let mut response = String::new();
                for (i, stat) in stats.iter().enumerate() {
                    response.push_str(&format!(
                        "__{}__ — *{}* Ls",
                        bot.get_chat_member(
                            msg.chat.id,
                            UserId(stat.tg_user_id.parse().into_diagnostic()?)
                        )
                        .await
                        .into_diagnostic()?
                        .user
                        .first_name,
                        stat.ls
                    ));
                    if i < stats.len() - 1 {
                        response.push_str("\n");
                    }
                }

                respond!(response);
            }
            Self::GiveL => {
                let Some(awardee) = msg
                    .reply_to_message()
                    .and_then(|m| m.from()) else {
                        info!("Message is not a reply");
                        respond!("reply to the person you want the L awarded to");
                    };

                let member = sqlx::query_as::<_, MemberStat>(
                    r#"
                    SELECT tgUserId, ls
                    FROM Member
                    LEFT JOIN Stat
                           ON Member.id = Stat.memberId
                    WHERE id = ?
                    "#,
                )
                .bind(awardee.id.to_string())
                .fetch_optional(db())
                .await
                .into_diagnostic()?;

                match member {
                    Some(member) => {
                        sqlx::query(
                            r#"
                            UPDATE Stat SET ls = ls + 1
                            WHERE memberId = ?
                            "#,
                        )
                        .bind(member.tg_user_id)
                        .execute(db())
                        .await
                        .into_diagnostic()?;
                    }
                    None => {
                        sqlx::query(
                            r#"
                            INSERT INTO Member (
                                id,
                                tgUserId
                            ) VALUES (?1, ?2);
                            
                            INSERT INTO Stat (
                                memberId,
                                ls
                            ) VALUES (?1, ?3);
                            "#,
                        )
                        .bind(awardee.id.to_string())
                        .bind(awardee.id.to_string())
                        .bind(1)
                        .execute(db())
                        .await
                        .into_diagnostic()?;
                    }
                }

                bot.send_message(msg.chat.id, "L has been awarded")
                    .parse_mode(ParseMode::MarkdownV2)
                    .reply_to_message_id(msg.id)
                    .await
                    .into_diagnostic()?;

                // TODO: Have bot delete message after x time has passed
                // bot.delete_message(msg.chat.id, sent.id)
                //     .await
                //     .into_diagnostic()
                //     .unwrap();
            }
            Self::Learn(body) => {
                let args = body.split(" | ").collect::<Vec<&str>>();
                let phrase = {
                    let Some(phrase) = args.get(0) else {
                        respond!("input not specified");
                    };
                    if phrase.is_empty() {
                        respond!("input not specified");
                    }
                    phrase
                };
                let Some(response) = args.get(1) else {
                    respond!("response not specified");
                };

                let nphrase = text::normalize(&phrase);

                let existing_phrase = sqlx::query_as::<_, Phrase>(
                    r#"
                    SELECT id, authorId, content
                    FROM Phrase
                    WHERE content = ?
                    "#,
                )
                .bind(phrase)
                .fetch_optional(db())
                .await
                .into_diagnostic()?;

                if let Some(phrase) = existing_phrase {
                    sqlx::query(
                        r#"
                        INSERT INTO Response (phraseId, content) VALUES (?, ?);
                        "#,
                    )
                    .bind(phrase.id)
                    .bind(&response)
                    .execute(db())
                    .await
                    .into_diagnostic()?;
                } else {
                    sqlx::query(
                        r#"
                        INSERT INTO Member (id, tgUserId)
                        SELECT ?1, ?1 WHERE NOT EXISTS (
                            SELECT 1 FROM Member
                            WHERE id = ?1
                              AND tgUserId = ?1
                        );
                        "#,
                    )
                    .bind(author.id.to_string())
                    .execute(db())
                    .await
                    .into_diagnostic()?;

                    sqlx::query(
                        r#"
                        INSERT INTO Phrase   (authorId, content) VALUES (?, ?);
                        INSERT INTO Response (phraseId, content) VALUES (last_insert_rowid(), ?);
                        "#,
                    )
                    .bind(author.id.to_string())
                    .bind(nphrase)
                    .bind(&response)
                    .execute(db())
                    .await
                    .into_diagnostic()?;
                }

                respond!("learnt");
            }
        }

        Ok(None)
    }
}

async fn command_handler(
    cmd: Command,
    bot: Bot,
    me: teloxide::types::Me,
    msg: Message,
) -> Result<()> {
    if let Some(response) = Command::handle(&cmd, bot.clone(), me, msg.clone()).await? {
        bot.send_message(msg.chat.id, response)
            .parse_mode(ParseMode::MarkdownV2)
            .reply_to_message_id(msg.id)
            .await
            .into_diagnostic()?;
    }

    Ok(())
}

async fn fallback_handler(bot: Bot, msg: Message) -> Result<()> {
    let Some(content) = msg.text() else {
        bail!("Message content not found");
    };
    let ncontent = text::normalize(content);

    let turns = sqlx::query_as::<_, DialogTurn>(
        r#"
        SELECT
            Phrase.content   AS phrase,
            Response.content AS response
        FROM Phrase
        LEFT JOIN Response
               ON Response.phraseId = Phrase.id
        WHERE phrase = ?1
        "#,
    )
    .bind(ncontent)
    .fetch_all(db())
    .await
    .into_diagnostic()?;

    if turns.is_empty() {
        bail!("No dialog matched");
    }

    let Some(turn) = turns.choose(&mut rand::thread_rng()) else {
        bail!("Failed to choose random dialog turn");
    };

    bot.send_message(msg.chat.id, &turn.response)
        .reply_to_message_id(msg.id)
        .await
        .into_diagnostic()?;

    Ok(())
}

fn schema() -> UpdateHandler<miette::Error> {
    Update::filter_message()
        // You can use branching to define multiple ways in which an update will be handled. If the
        // first branch fails, an update will be passed to the second branch, and so on.
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(command_handler),
        )
        .branch(
            dptree::filter_map_async(|msg: Message, me: teloxide::types::Me| async move {
                debug!("Incoming text message: {:#?}", msg);

                let mut input;
                let Some(text) = msg.text() else {
                    return None
                };
                input = text.to_string();
                if text.is_empty() {
                    // handle reply
                    match msg.kind {
                        MessageKind::Common(msg) => match msg.media_kind {
                            MediaKind::Text(media) => input = media.text,
                            _ => unimplemented!(),
                        },
                        _ => unimplemented!(),
                    }
                }
                Command::from_phrase(input, me).await
            })
            .endpoint(command_handler),
        )
        .branch(Message::filter_text().endpoint(fallback_handler))
}

pub async fn run_bot() -> Result<()> {
    let token = env::var("TELEGRAM_API_TOKEN")
        .into_diagnostic()
        .wrap_err("TELEGRAM_API_TOKEN not found in environment")?;
    let bot = Bot::new(token);

    bot.set_my_commands(Command::bot_commands())
        .await
        .into_diagnostic()?;

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![])
        .default_handler(|update| async move {
            warn!("Unhandled update: {:?}", update);
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
