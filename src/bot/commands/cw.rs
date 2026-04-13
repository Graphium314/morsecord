use anyhow::Context as _;
use serenity::all::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::Context;

use crate::bot::commands::get_value_f64;

impl crate::bot::Bot {
    pub async fn run_command_speed(
        &self,
        _ctx: &Context,
        command: &CommandInteraction,
    ) -> anyhow::Result<String> {
        let new_speed = command
            .data
            .options()
            .iter()
            .find(|option| option.name == "speed")
            .map(|option| get_value_f64(&option.value))
            .context("no argument")??;

        sqlx::query("insert into cw_speed (id, speed) values (?, ?) on conflict (id) do update set speed = excluded.speed")
            .bind(command.user.id.to_string())
            .bind(new_speed)
            .execute(&self.db)
            .await
            .context("internal error")?;

        Ok("ok!".to_string())
    }

    pub async fn run_command_freq(
        &self,
        _ctx: &Context,
        command: &CommandInteraction,
    ) -> anyhow::Result<String> {
        let new_freq = command
            .data
            .options()
            .iter()
            .find(|option| option.name == "freq")
            .map(|option| get_value_f64(&option.value))
            .context("no argument")??;

        sqlx::query("insert into cw_speed (id, freq) values (?, ?) on conflict (id) do update set freq = excluded.freq")
            .bind(command.user.id.to_string())
            .bind(new_freq)
            .execute(&self.db)
            .await
            .context("internal error")?;

        Ok("ok!".to_string())
    }

    pub fn register_commands_cw() -> [CreateCommand; 2] {
        [
            CreateCommand::new("cw-speed")
                .description("set cw speed")
                .add_option(
                    CreateCommandOption::new(CommandOptionType::Number, "speed", "speed(wpm)")
                        .min_number_value(5.0)
                        .required(true),
                ),
            CreateCommand::new("cw-freq")
                .description("set cw freq")
                .add_option(
                    CreateCommandOption::new(CommandOptionType::Number, "freq", "freq (Hz)")
                        .min_number_value(10.0)
                        .required(true),
                ),
        ]
    }
}
