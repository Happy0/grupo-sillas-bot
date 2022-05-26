use crate::discord_bot_types;
use crate::lol;
use std::env;

pub async fn execute_played_command(played_command: discord_bot_types::Command) -> Result<String, discord_bot_types::BotError> {
    
    // TODO: instantiate in main handler and pass this in
    let api_fetcher: lol::api_fetcher::BoundedHttpFetcher = lol::api_fetcher::create_lol_client(20,100);

    println!("Executing played command");
    let x = build_played_command(played_command.options)?;

    let api_key = env::var("LOL_API_KEY").map_err(|err| discord_bot_types::BotError {
        statusCode: 500,
        body: "Missing LOL API key".to_string()
    })?;

    return Ok("I don't do anything yet... I will return a wee recent game summary".to_string());
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