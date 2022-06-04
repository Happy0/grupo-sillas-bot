use crate::discord_bot_types;

pub fn build_played_command(command: discord_bot_types::Command, token: String, application_id: String) -> Result<discord_bot_types::PlayedCommand, discord_bot_types::BotError> {
    let player_name = command.options.iter().find_map(|x| match x {
        discord_bot_types::CommandOption::NumberCommandOption(x) => None,
        discord_bot_types::CommandOption::StringCommandOption(option) => {
            if option.name == "user" {Some(&option.value)} else {None}
        },
    }).ok_or(discord_bot_types::BotError {
        statusCode: 500,
        body: "Could not find player name".to_string()
    })?;

    let days_requested = command.options.iter().find_map(|x| match x {
        discord_bot_types::CommandOption::NumberCommandOption(option) => {if option.name == "days" {Some(option.value)} else {None}},
        discord_bot_types::CommandOption::StringCommandOption(option) => None,
    }).ok_or(discord_bot_types::BotError {
        statusCode: 500,
        body: "Could not find player name".to_string()
    })?;

    let game_type = if command.name == "ranked" {Some("ranked".to_string())} else {None};

    return Ok(discord_bot_types::PlayedCommand {
        player_name: player_name.to_string(),
        days: days_requested,
        game_type: game_type,
        token: token,
        application_id: application_id
    });
}
