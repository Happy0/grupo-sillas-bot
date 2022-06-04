use lol;
use std::env;
use common::discord_bot_types;
use crate::models;

pub async fn execute_played_command(
    lol_api_fetcher: &lol::api_fetcher::BoundedHttpFetcher,
    command: discord_bot_types::PlayedCommand
    ) -> Result<String, discord_bot_types::BotError> {
    
    println!("Executing played command");

    let days = if command.days > 7 {7} else { command.days };

    let api_key: String = env::var("LOL_API_KEY").map_err(|err| discord_bot_types::BotError {
        statusCode: 500,
        body: "Missing LOL API key".to_string()
    })?;

    let puuid = lol::get_puuid(&lol_api_fetcher, "euw1", &command.player_name, &api_key).await.map_err(models::to_bot_error)?;
    let game_ids = lol::get_game_ids(&lol_api_fetcher, &api_key, "europe", &puuid, days, &command.game_type).await.map_err(models::to_bot_error)?;
    let models = lol::fetch_game_summaries(&lol_api_fetcher, &api_key, "europe", &puuid, game_ids).await.map_err(models::to_bot_error)?;

    let played_for: u64 = calculate_time_played(&models);
    let time_played_string: String = create_time_played_string(played_for);
    let wins = calculate_wins(&models);
    let loses = calculate_loses(&models);

    let game_summary_strings = models.iter().map(create_game_stats_string);

    let mut message= format!("{} has played for {} over {} days\nThey won {} games and lost {}", command.player_name, time_played_string, command.days, wins, loses).to_string();
    message.push_str("\n");
    for summary in game_summary_strings {
        message.push_str(&summary);
        message.push_str("\n");
    }

    return Ok(message);
}

fn calculate_time_played(summaries: &Vec<lol::models::UserGameSummary>) -> u64 {
    return summaries.iter().map(|x| x.game_duration_millis).sum();
}

fn calculate_wins(summaries: &Vec<lol::models::UserGameSummary>) -> u64 {
    return summaries.iter().map(|x| if x.participant.win == true {1} else {0}).sum();
}

fn calculate_loses(summaries: &Vec<lol::models::UserGameSummary>) -> u64 {
    return summaries.iter().map(|x| if x.participant.win == true {0} else {1}).sum();
}

fn create_time_played_string(millis: u64) -> String {
    println!("millis: {}", millis);
    let seconds = millis / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    
    let minutes = minutes % 60;
    return format!("{} hours and {} minutes", hours, minutes);
}

fn create_game_stats_string(game_summary: &lol::models::UserGameSummary) -> String {
    let participant = &game_summary.participant;
    let full_info_url = format!("https://www.leagueofgraphs.com/match/euw/{}#participant1", game_summary.game_id);

    let win_or_loss = if game_summary.participant.win {"Win"} else {"Loss"};

    return format!("[{}] {}/{}/{} ({}) <{}>", participant.championName, participant.kills, participant.deaths, participant.assists, win_or_loss, full_info_url);
}
