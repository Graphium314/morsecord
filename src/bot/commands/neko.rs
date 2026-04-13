use crate::bot::commands::get_value_i64;
use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption, ResolvedOption};

impl crate::bot::Bot {
    pub fn register_command_neko() -> CreateCommand {
        CreateCommand::new("neko")
            .description("猫のように鳴く")
            .add_option(
                CreateCommandOption::new(CommandOptionType::Integer, "count", "にゃーんの回数")
                    .min_int_value(1)
                    .max_int_value(32)
                    .required(false),
            )
    }

    pub fn run_command_neko(&self, options: &[ResolvedOption]) -> anyhow::Result<String> {
        let count = options
            .iter()
            .find(|option| option.name == "count")
            .map(|option| get_value_i64(&option.value))
            .unwrap_or(Ok(1))?;

        let count = count.clamp(1, 32);

        Ok("にゃーん".to_string().repeat(count as usize))
    }
}
