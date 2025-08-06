use anyhow::Context as _;
use serenity::{
    model::application::{
        command::Command, interaction::application_command::ApplicationCommandInteraction,
    },
    prelude::Context,
};
use std::sync::{Arc, Mutex};

use crate::{
    bot::BotStateMode,
    modes::lesson_long::{start, LessonLongModeState},
};



impl crate::bot::Bot {
    pub async fn run_command_lesson_long_start(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> anyhow::Result<String> {
        let mut probset = "alpha".to_string();
        let mut length = None;
        let mut speed = None;
        let mut freq = None;

        for option in &command.data.options {
            let v = option.value.as_ref().context("value empty")?;
            let vf = v.as_f64().map(|x| x as f32).context("value is not f64");
            let vs = v.as_str().context("value is not string");

            match option.name.as_str() {
                "speed" => speed = Some(vf?),
                "freq" => freq = Some(vf?),
                "probset" => probset = vs?.to_string(),
                "length" => {
                    length = Some(v
                        .as_f64().context("value is not f64")? as usize);
                }
                _ => (),
            }
        }

        let speed = speed.unwrap_or(20.0);
        let freq = freq.unwrap_or(800.0);
        let length = length.unwrap_or(10);

        probset.make_ascii_lowercase();

        let gid = command.guild_id.context("not in guild")?;
        let state = Arc::new(Mutex::new(LessonLongModeState::new(
            probset.as_str(),
            length,
            speed,
            freq,
        )?));
        start(ctx, gid, state.clone())
            .await
            .context("internal error")?;
        self.switch_mode(gid.0, BotStateMode::LessonLong(state))?;

        Ok("let's start long lesson".to_string())
    }

    pub async fn run_command_lesson_long_end(
        &self,
        _ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> anyhow::Result<String> {
        let r = self.switch_mode(
            command.guild_id.context("no guild")?.0,
            BotStateMode::Normal,
        )?;
        Ok(r)
    }

    pub async fn register_commands_cw_lesson_long(&self, ctx: &Context) -> anyhow::Result<()> {
        Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("cw-start-long-lesson")
                .description("start long lesson")
                .create_option(|option| {
                    option
                        .name("probset")
                        .description("problem set name")
                        .kind(serenity::model::prelude::command::CommandOptionType::String)
                        .required(false)
                })
                .create_option(|option| {
                    option
                        .name("length")
                        .description("problem length")
                        .kind(serenity::model::prelude::command::CommandOptionType::Number)
                        .required(false)
                })
                .create_option(|option| {
                    option
                        .name("speed")
                        .description("speed in WPM")
                        .kind(serenity::model::prelude::command::CommandOptionType::Number)
                        .min_number_value(5.0)
                        .required(false)
                })
                .create_option(|option| {
                    option
                        .name("freq")
                        .description("frequency in Hz")
                        .kind(serenity::model::prelude::command::CommandOptionType::Number)
                        .min_number_value(200.0)
                        .required(false)
                })
        })
        .await
        .context("command cw-start-long-lesson registration failed")?;

        Ok(())
    }
}
