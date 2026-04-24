mod commands;

use std::sync::{Arc, Mutex};

use serenity::all::{
    ActivityData, Command, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse, EventHandler, Interaction, Message, Ready,
};
use serenity::async_trait;

use anyhow::Context as _;

#[derive(Clone, Default)]
pub enum BotStateMode {
    #[default]
    Normal,
    Lesson(Arc<Mutex<crate::modes::lesson::LessonModeState>>),
    LessonLong(Arc<Mutex<crate::modes::lesson_long::LessonLongModeState>>),
}

impl BotStateMode {
    // NOTE: call this when you want to discard the state
    // Drop trait is not implemented because BotStateMode is cloned at Bot::message. :(
    pub fn discard(&self) -> Option<String> {
        match self {
            BotStateMode::Normal => None,
            BotStateMode::Lesson(s) => {
                log::info!("terminating callsign lesson");
                crate::modes::lesson::end(s.clone()).ok()
            }
            BotStateMode::LessonLong(s) => {
                log::info!("terminating long lesson");
                crate::modes::lesson_long::end(s.clone()).ok()
            }
        }
    }
}

struct BotState {
    txt_ch: serenity::model::id::ChannelId,
    mode: Arc<Mutex<BotStateMode>>,
}

impl Clone for BotState {
    fn clone(&self) -> Self {
        BotState {
            txt_ch: self.txt_ch,
            mode: self.mode.clone(),
        }
    }
}

pub struct Bot {
    db: sqlx::SqlitePool,

    states: Arc<Mutex<std::collections::HashMap<u64, BotState>>>,
}

impl Bot {
    pub async fn new(db: sqlx::SqlitePool) -> Self {
        Bot {
            db,
            states: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    // called when bot joins to a call
    pub fn add_call_state(
        &self,
        guild_id: u64,
        ch: serenity::model::prelude::ChannelId,
    ) -> anyhow::Result<()> {
        let mut states = self
            .states
            .lock()
            .or_else(|_| anyhow::bail!("lock failed"))
            .context("internal error")?;

        anyhow::ensure!(!states.contains_key(&guild_id), "already in call");

        states.insert(
            guild_id,
            BotState {
                txt_ch: ch,
                mode: Arc::new(Mutex::new(BotStateMode::Normal)),
            },
        );

        Ok(())
    }

    pub fn switch_mode(&self, guild_id: u64, mode: BotStateMode) -> anyhow::Result<String> {
        let mut mode = Arc::new(Mutex::new(mode));
        std::mem::swap(
            &mut self
                .states
                .lock()
                .or_else(|_| anyhow::bail!("lock failed"))
                .context("internal error")?
                .get_mut(&guild_id)
                .context("not in call")?
                .mode,
            &mut mode,
        );

        let r = mode
            .lock()
            .or_else(|_| anyhow::bail!("lock failed"))
            .context("internal error")?
            .discard()
            .unwrap_or_else(|| "".to_string());

        Ok(r)
    }

    pub fn get_call_txt_ch(&self, guild_id: u64) -> anyhow::Result<serenity::model::id::ChannelId> {
        let states = self
            .states
            .lock()
            .or_else(|_| anyhow::bail!("lock failed"))
            .context("internal error")?;

        Ok(states.get(&guild_id).context("not in call")?.txt_ch)
    }

    pub fn get_call_mode(&self, guild_id: u64) -> anyhow::Result<Arc<Mutex<BotStateMode>>> {
        let states = self
            .states
            .lock()
            .or_else(|_| anyhow::bail!("lock failed"))
            .context("internal error")?;

        Ok(states.get(&guild_id).context("not in call")?.mode.clone())
    }

    pub fn erase_call_state(&self, guild_id: u64) -> anyhow::Result<()> {
        let mut states = self
            .states
            .lock()
            .or_else(|_| anyhow::bail!("lock failed"))
            .context("internal error")?;

        let state = states.remove(&guild_id).context("not in call")?;

        state
            .mode
            .lock()
            .or_else(|_| anyhow::bail!("lock failed"))
            .context("internal error")?
            .discard();

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Bot {
    // Botが起動したときに走る処理
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(Some(ActivityData::listening("7.074MHz")));
        log::info!("{} is connected!", ready.user.name);

        let mut cmds = vec![];
        cmds.push(Self::register_command_neko());
        cmds.extend_from_slice(&Self::register_commands_vc());
        cmds.extend_from_slice(&Self::register_commands_cw());
        cmds.extend_from_slice(&Self::register_commands_cw_lesson());
        cmds.push(Self::register_commands_cw_lesson_long());
        let r = Command::set_global_commands(&ctx.http, cmds).await;
        if let Err(why) = r {
            log::error!("Couldn't register slash commands: {}", why);
        }

        log::info!("commands registered");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            log::info!("got command: {}", command.data.name);

            // VC commands can take >3s (DAVE handshake), so defer immediately
            let needs_defer = matches!(command.data.name.as_str(), "cw-join" | "cw-leave");

            if needs_defer {
                let defer = CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new());
                if let Err(why) = command.create_response(&ctx.http, defer).await {
                    log::error!("Cannot defer slash command: {}", why);
                    return;
                }
            }

            let content = match command.data.name.as_str() {
                "neko" => self.run_command_neko(&command.data.options()),
                "cw-join" => self.run_command_join(&ctx, &command).await,
                "cw-leave" => self.run_command_leave(&ctx, &command).await,
                "cw-speed" => self.run_command_speed(&ctx, &command).await,
                "cw-freq" => self.run_command_freq(&ctx, &command).await,
                "cw-start-lesson" => self.run_command_lesson_start(&ctx, &command).await,
                "cw-end-lesson" => self.run_command_lesson_end(&ctx, &command).await,
                "cw-start-long-lesson" => self.run_command_lesson_long_start(&ctx, &command).await,
                _ => Ok("not implemented :(".to_string()),
            }
            .unwrap_or_else(|e| {
                log::error!("{:#}", e);
                e.to_string()
            });

            if content.is_empty() {
                return;
            }

            if needs_defer {
                let edit = EditInteractionResponse::new().content(content);
                if let Err(why) = command.edit_response(&ctx.http, edit).await {
                    log::error!("Cannot edit slash command response: {}", why);
                }
            } else {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    log::error!("Cannot respond to slash command: {}", why);
                }
            }
        }
    }

    async fn message(&self, ctx: Context, message: Message) {
        if message.author.bot {
            return;
        }

        let Some(gid) = message.guild_id else {
            log::info!("message not in guild");
            return;
        };

        let Ok(cid) = self.get_call_txt_ch(gid.into()) else {
            return;
        };

        if cid != message.channel_id {
            return;
        }

        let mode = match self.get_call_mode(gid.into()) {
            Ok(m) => m,
            Err(e) => {
                log::error!("{:#}", e);
                return;
            }
        };

        log::info!("got message: {}", message.content);

        // NOTE: actually clone not needed but compiler complains: https://github.com/rust-lang/rust/issues/104883
        let mode = (*mode.lock().unwrap()).clone();
        match mode {
            BotStateMode::Normal => {
                crate::modes::normal::on_message(&ctx, &message, &self.db).await
            }

            BotStateMode::Lesson(s) => {
                crate::modes::lesson::on_message(&ctx, &message, s.clone()).await
            }

            BotStateMode::LessonLong(s) => {
                crate::modes::lesson_long::on_message(&ctx, &message, s.clone()).await
            }
        }
        .unwrap_or_else(|e| {
            log::error!("{:#}", e);
        });
    }
}
