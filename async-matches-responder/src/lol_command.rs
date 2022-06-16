use lol;
use std::env;
use common::discord_bot_types;
use crate::models;

pub async fn execute_played_command(
    lol_api_fetcher: &lol::api_fetcher::BoundedHttpFetcher,
    command: &discord_bot_types::PlayedCommand
    ) -> Result<String, discord_bot_types::BotError> {

    let days = if command.days > 7 {7} else { command.days };

    let api_key: String = env::var("LOL_API_KEY").map_err(|err| discord_bot_types::BotError {
        statusCode: 500,
        body: "Missing LOL API key".to_string()
    })?;

    match &command.game_type {
        Some(mode) if mode == "ranked" => get_ranked_games_summary(lol_api_fetcher, &command.player_name, days).await,
        None => get_all_games_summary(lol_api_fetcher, &command.player_name, days).await,
        Some(mode) => Ok(format!("Unrecognised game mode {}", mode))
    }
}

pub async fn get_ranked_games_summary(lol_api_fetcher: &lol::api_fetcher::BoundedHttpFetcher, player_name: &str, days: u64) -> Result<String, discord_bot_types::BotError> {
    let api_key = get_api_key()?;
    let summary = get_games_over_time(lol_api_fetcher, &api_key, player_name, days, Some("ranked".to_string()));
    let ranked_sum = get_current_rank(lol_api_fetcher, &api_key, player_name);

    let (game_summaries, ranked_summary) = tokio::try_join!(summary, ranked_sum)?;
    let time_played_string: String = create_time_played_string(game_summaries.played_for_millis);

    match ranked_summary {
        None => Ok(format!("{} has not played any ranked games.", player_name)),
        Some(ranked_summary) => {
            let mut message = format!("{} ({} {}, {} points) has played for {} over {} days\nThey won {} games and lost {}", player_name, ranked_summary.rank, ranked_summary.tier, ranked_summary.leaguePoints, time_played_string, days, game_summaries.wins, game_summaries.losses).to_string();
            message.push_str("\n");
            message.push_str(&create_summaries_string(game_summaries.games));
        
            return Ok(message);
        }
    }
}

pub async fn get_all_games_summary(lol_api_fetcher: &lol::api_fetcher::BoundedHttpFetcher, player_name: &str, days: u64) -> Result<String, discord_bot_types::BotError> {
    let api_key = get_api_key()?;
    let game_summaries = get_games_over_time(lol_api_fetcher, &api_key, player_name, days, None).await?;
    let time_played_string: String = create_time_played_string(game_summaries.played_for_millis);

    let mut message = format!("{} has played for {} over {} days\nThey won {} games and lost {}", player_name, time_played_string, days, game_summaries.wins, game_summaries.losses).to_string();
    message.push_str("\n");
    message.push_str(&create_summaries_string(game_summaries.games));

    return Ok(message);
}

fn create_summaries_string(summaries: Vec<lol::models::UserGameSummary>) -> String {
    let mut result = "".to_string();
    for summary in summaries.iter().take(10) {
        let game_summary_string = create_game_stats_string(&summary);
        result.push_str(&game_summary_string);
        result.push_str("\n");
    }
    return result;
}

pub async fn get_current_rank(
    lol_api_fetcher: &lol::api_fetcher::BoundedHttpFetcher,
    api_key: &str,
    player_name: &str) -> Result<Option<lol::models::LeagueEntry>, discord_bot_types::BotError> {

    let summoner_id = lol::get_encrypted_summoner_id(lol_api_fetcher, "euw1", player_name, &api_key).await.map_err(models::to_bot_error)?;
    return lol::get_solo_queue_ranking(lol_api_fetcher, "euw1", summoner_id, &api_key).await.map_err(models::to_bot_error);
}

fn get_api_key() -> Result<String, discord_bot_types::BotError> {
    return env::var("LOL_API_KEY").map_err(|err| discord_bot_types::BotError {
        statusCode: 500,
        body: "Missing LOL API key".to_string()
    });
}

async fn get_games_over_time(
    lol_api_fetcher: &lol::api_fetcher::BoundedHttpFetcher,
    api_key: &str,
    player_name: &str,
    days: u64,
    game_type: Option<String>) ->  Result<models::GamesOverTimeSummary, discord_bot_types::BotError> {
    
    println!("Executing get_games_over_time");

    let days = if days > 7 { 7 } else { days };

    let api_key: String = env::var("LOL_API_KEY").map_err(|err| discord_bot_types::BotError {
        statusCode: 500,
        body: "Missing LOL API key".to_string()
    })?;

    let puuid = lol::get_puuid(&lol_api_fetcher, "euw1", player_name, &api_key).await.map_err(models::to_bot_error)?;
    let game_ids = lol::get_game_ids(&lol_api_fetcher, &api_key, "europe", &puuid, days, &game_type).await.map_err(models::to_bot_error)?;
    let models = lol::fetch_game_summaries(&lol_api_fetcher, &api_key, "europe", &puuid, game_ids).await.map_err(models::to_bot_error)?;

    let played_for: u64 = calculate_time_played(&models);
    let wins = calculate_wins(&models);
    let loses = calculate_loses(&models);

    let result = models::GamesOverTimeSummary {
        games: models,
        wins: wins,
        losses: loses,
        played_for_millis: played_for 
    };
    
    Ok(result)
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
