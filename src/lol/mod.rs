use std::collections::HashMap;
use reqwest;
use serde_json::{Value};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use futures::future::join_all;
use std::env;

mod api_fetcher;
mod models;

/**
 * Returns the list of IDs of the games the given player (by puuid) has played in over the given period of days
 */
pub async fn get_game_ids(client: &api_fetcher::BoundedHttpFetcher, api_key: &str, region: &str, puuid: &str, days: u64) -> Result<Vec<String>, models::LolApiError> {

    let mut game_ids: std::vec::Vec<String> = Vec::new();
    let mut start_index: usize = 0;
    let page_size: usize = 100;
    loop {
        let request_url = build_game_ids_request_url(region, api_key, puuid, days, start_index, page_size);
        println!("{}", request_url);

        let res = api_fetcher::get_request(client, request_url).await?;

        let status = res.status();

        if !status.is_success() {
            return Err(models::LolApiError {description: format!("Erroring fetching game IDs list, status code: {}", res.status()), http_code: res.status().to_string()})
        } 

        let result_list = res.json::<Vec<String>>().await;

        match result_list {
            Ok(results) => {
                game_ids.extend_from_slice(&results);

                if results.len() == 0 || results.len() < page_size {
                    // No more results to page through
                    break;
                }

                start_index = start_index + page_size;
            },
            Err(_error) => {
                return Err(models::LolApiError {description: format!("Unexpectedly not a JSON body, HTTP code: {}.", status.as_str()), http_code: status.as_str().to_string()})
            }
        }
    }

    return Ok(game_ids);
}

async fn fetch_game_summaries(client: &api_fetcher::BoundedHttpFetcher, api_key: &str, region: &str, puuid: &str, game_ids: Vec<String>) -> Result<Vec<models::UserGameSummary>, models::LolApiError> {
    let futures = game_ids.iter().map(|game_id| get_game_player_summary(client, region, game_id, puuid, api_key) );
    let y = join_all(futures).await;
    return y.into_iter().collect::<Result<Vec<_>,_>>();
}

async fn get_game_player_summary(client: &api_fetcher::BoundedHttpFetcher, region: &str, game_id: &str, puuid: &str, api_key: &str) -> Result<models::UserGameSummary, models::LolApiError> {
    let request_url = format!("https://{}.api.riotgames.com/lol/match/v5/matches/{}?api_key={}", region, game_id, api_key);

    let res = api_fetcher::get_request(client, request_url).await?;

    if !res.status().is_success() {
        return Err(models::LolApiError {description: "HTTP error getting user summary in match".to_string(), http_code: res.status().as_str().to_string() })
    }

    let body = res.json::<models::GameSummary>().await?;
    let user = find_user_game_summary(puuid, body.info.participants);

    match user {
        None => return Err(models::LolApiError {description: "Could not find user summary in match".to_string(), http_code: "500".to_string()}),
        Some(participant) => return Ok(models::UserGameSummary {participant: participant, game_duration_millis: body.info.gameDuration})
    }

}

fn find_user_game_summary(puuid: &str, participants: Vec<models::Participant>) -> Option<models::Participant> {
    return participants.into_iter().find(|x| x.puuid == puuid );
}

fn build_game_ids_request_url(
    region: &str,
    api_key: &str,
    puuid: &str,
    days: u64,
    start_index: usize,
    page_size: usize) -> String {
    
    let now = SystemTime::now();
    let end_time = now
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let start_time = end_time - Duration::new(days * 86400, 0) ;

    return format!("https://{}.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids?api_key={}&count={}&start={}&startTime={}&endTime={}", region, puuid, api_key, page_size, start_index, start_time.as_secs(), end_time.as_secs());
}

async fn get_puuid(client: &api_fetcher::BoundedHttpFetcher, region: &str, user_name: &str, api_key: &str) -> Result<String, models::LolApiError> {
    let request_url =  format!("https://{}.api.riotgames.com/lol/summoner/v4/summoners/by-name/{}?api_key={}", region, user_name, api_key);

    let res = api_fetcher::get_request(client, request_url).await?;

    let status_code = res.status();
    
    let body = res.json::<HashMap<String, Value>>().await?;
    let result = body.get("puuid").and_then(Value::as_str);

    return match result {
         Some(x) => Ok(x.to_string()),
         None => Err(models::LolApiError {description: format!("Unexpected no 'puuid' in response body. HTTP status code: {}", status_code), http_code: status_code.to_string()})
    }
}

