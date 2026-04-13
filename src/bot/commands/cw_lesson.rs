use anyhow::Context as _;
use serenity::all::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::Context;
use std::sync::{Arc, Mutex};

use crate::bot::commands::{get_value_f64, get_value_str};
use crate::bot::BotStateMode;

impl crate::bot::Bot {
    pub async fn run_command_lesson_start(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
    ) -> anyhow::Result<String> {
        let mut min_speed = None;
        let mut max_speed = None;
        let mut min_freq = None;
        let mut max_freq = None;
        let mut probset = "call_ja".to_string();

        command
            .data
            .options()
            .iter()
            .try_fold::<_, _, anyhow::Result<()>>((), |_, x| {
                let vf = get_value_f64(&x.value).map(|x| x as f32);
                let vs = get_value_str(&x.value);

                match x.name {
                    "min_speed" => min_speed = Some(vf?),
                    "max_speed" => max_speed = Some(vf?),
                    "min_freq" => min_freq = Some(vf?),
                    "max_freq" => max_freq = Some(vf?),
                    "probset" => probset = vs?.to_string(),
                    _ => (),
                };
                Ok(())
            })?;

        let min_speed = min_speed.unwrap_or(15.0_f32.min(max_speed.unwrap_or(f32::NAN)));
        let max_speed = max_speed.unwrap_or(20.0_f32.max(min_speed));

        anyhow::ensure!(min_speed <= max_speed, "min_speed > max_speed");

        let min_freq = min_freq.unwrap_or(500.0_f32.min(max_freq.unwrap_or(f32::NAN)));
        let max_freq = max_freq.unwrap_or(1000.0_f32.max(min_freq));

        anyhow::ensure!(min_freq <= max_freq, "min_freq > max_freq");

        let speed_range = min_speed..=max_speed;
        let freq_range = min_freq..=max_freq;

        probset.make_ascii_lowercase();

        let gid = command.guild_id.context("not in guild")?;
        let state = Arc::new(Mutex::new(crate::modes::lesson::LessonModeState::new(
            speed_range,
            freq_range,
            probset.as_str(),
        )?));
        crate::modes::lesson::start(ctx, gid, state.clone())
            .await
            .context("internal error")?;
        self.switch_mode(gid.into(), BotStateMode::Lesson(state))?;

        Ok("let's start lesson".to_string())
    }

    pub async fn run_command_lesson_end(
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

    pub fn register_commands_cw_lesson() -> [CreateCommand; 2] {
        [
            CreateCommand::new("cw-start-lesson")
                .description("start callsign lesson")
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::Number,
                        "min_speed",
                        "minimum speed",
                    )
                    .min_number_value(5.0)
                    .required(false),
                )
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::Number,
                        "max_speed",
                        "maximum speed",
                    )
                    .min_number_value(5.0)
                    .required(false),
                )
                .add_option(
                    CreateCommandOption::new(CommandOptionType::Number, "min_freq", "minimum freq")
                        .min_number_value(200.0)
                        .required(false),
                )
                .add_option(
                    CreateCommandOption::new(CommandOptionType::Number, "max_freq", "maximum freq")
                        .min_number_value(200.0)
                        .required(false),
                )
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "probset",
                        "problem set name (can be followed by colon and args)",
                    )
                    .required(false),
                ),
            CreateCommand::new("cw-end-lesson").description("end callsign lesson"),
        ]
    }
}
