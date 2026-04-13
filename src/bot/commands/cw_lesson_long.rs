use anyhow::Context as _;
use serenity::all::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::Context;
use std::sync::{Arc, Mutex};

use crate::bot::commands::{get_value_f64, get_value_str};
use crate::{
    bot::BotStateMode,
    modes::lesson_long::{start, LessonLongModeState},
};

impl crate::bot::Bot {
    pub async fn run_command_lesson_long_start(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
    ) -> anyhow::Result<String> {
        let mut probset = "alpha".to_string();
        let mut length = None;
        let mut speed = None;
        let mut freq = None;

        for option in &command.data.options() {
            let v = &option.value;
            let vf = get_value_f64(v)
                .map(|x| x as f32)
                .context("value is not f64");
            let vs = get_value_str(v).context("value is not str");

            match option.name {
                "speed" => speed = Some(vf?),
                "freq" => freq = Some(vf?),
                "probset" => probset = vs?.to_string(),
                "length" => length = Some(vf? as usize),
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
        self.switch_mode(gid.into(), BotStateMode::LessonLong(state))?;

        Ok("let's start long lesson".to_string())
    }

    pub async fn run_command_lesson_long_end(
        &self,
        _ctx: &Context,
        command: &CommandInteraction,
    ) -> anyhow::Result<String> {
        let r = self.switch_mode(
            command.guild_id.context("no guild")?.into(),
            BotStateMode::Normal,
        )?;
        Ok(r)
    }

    pub fn register_commands_cw_lesson_long() -> CreateCommand {
        CreateCommand::new("cw-start-long-lesson")
            .description("start long lesson")
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "probset", "problem set name")
                    .required(false),
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::Number, "length", "problem length")
                    .required(false),
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::Number, "speed", "speed in WPM")
                    .min_number_value(5.0)
                    .required(false),
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::Number, "freq", "frequency in Hz")
                    .min_number_value(200.0)
                    .required(false),
            )
    }
}
