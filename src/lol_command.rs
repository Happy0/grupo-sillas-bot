use crate::discord_bot_types;
use crate::lol;

pub async fn execute_played_command(played_command: discord_bot_types::Command) -> String {
    let x = build_played_command(played_command.options);

    match x {
        Err(x) => "Internal error: could not parse command".to_string(),
        Ok(x) => "I'd be about to execute the lol command...".to_string()
    }
}

fn build_played_command(command_options: Vec<discord_bot_types::CommandOption>) -> Result<discord_bot_types::PlayedCommand, discord_bot_types::BotError> {
    let player_name = command_options.iter().find_map(|x| match x {
        discord_bot_types::CommandOption::NumberCommandOption(x) => None,
        discord_bot_types::CommandOption::StringCommandOption(option) => {
            if option.name == "user" {Some(&option.value)} else {None}
        },
    }).ok_or(discord_bot_types::BotError {
        statusCode: 500,
        body: "Could not find player name".to_string()
    })?;

    let days_requested = command_options.iter().find_map(|x| match x {
        discord_bot_types::CommandOption::NumberCommandOption(option) => {if option.name == "days" {Some(option.value)} else {None}},
        discord_bot_types::CommandOption::StringCommandOption(option) => None,
    }).ok_or(discord_bot_types::BotError {
        statusCode: 500,
        body: "Could not find player name".to_string()
    })?;

    return Ok(discord_bot_types::PlayedCommand {
        player_name: player_name.to_string(),
        days: days_requested
    });
}