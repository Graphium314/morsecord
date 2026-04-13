use anyhow::Context as _;
use serenity::all::{CreateCommand, CreateCommandOption, PartialChannel};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::Context;

use crate::bot::commands::get_value_channel;

fn get_ch(cmd: &CommandInteraction) -> anyhow::Result<PartialChannel> {
    let ch = cmd
        .data
        .options()
        .iter()
        .find(|opt| opt.name == "ch")
        .map(|v| get_value_channel(&v.value).cloned())
        .expect("no ch option")?;

    Ok(ch)
}

impl crate::bot::Bot {
    pub async fn run_command_join(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
    ) -> anyhow::Result<String> {
        let ch = get_ch(command)?;
        let gid = command.guild_id.context("not in guild")?;

        self.add_call_state(gid.into(), command.channel_id)?;

        let man = songbird::get(ctx).await.expect("init songbird").clone();
        let handler = man.join(gid, ch.id).await.context("join failed")?;

        {
            let mut handler = handler.lock().await;
            handler.deafen(true).await.context("deafen failed")?;
        }

        Ok("got it!".to_string())
    }

    pub async fn run_command_leave(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
    ) -> anyhow::Result<String> {
        let man = songbird::get(ctx).await.expect("init songbird").clone();
        let gid = command.guild_id.context("not in guild")?;
        let _cid = command.channel_id;
        let has_handler = man.get(gid).is_some();

        self.erase_call_state(gid.into())?;

        if has_handler {
            man.remove(gid).await.context("leave failed")?;
        } else {
            return Ok("Not in a voice channel".into());
        }

        Ok("bye!".to_string())
    }

    pub fn register_commands_vc() -> [CreateCommand; 3] {
        [
            CreateCommand::new("cw-join")
                .description("join")
                .add_option(
                    CreateCommandOption::new(CommandOptionType::Channel, "ch", "channel to join")
                        .channel_types(vec![serenity::all::ChannelType::Voice])
                        .required(true),
                ),
            CreateCommand::new("cw-leave").description("leave"),
            CreateCommand::new("cw-play")
                .description("play")
                .add_option(
                    CreateCommandOption::new(CommandOptionType::String, "str", "string to play")
                        .required(true),
                ),
        ]
    }
}
