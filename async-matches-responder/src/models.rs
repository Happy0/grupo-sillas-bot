use lol;
use common::discord_bot_types;
use serde::{Deserialize, Serialize};

pub struct Toolbox {
    pub lol_api_fetcher: lol::api_fetcher::BoundedHttpFetcher,
    pub discord_http_client: reqwest::Client
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueBatch {
    pub Records: Vec<QueueRecord>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueRecord {
    pub body: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiscordResponseBody {
    pub content: String
}

pub struct GamesOverTimeSummary {
    pub games: Vec<lol::models::UserGameSummary>,
    pub wins: u64,
    pub losses: u64,
    pub played_for_millis: u64
}

/**
 * Converts between an error received from the LoL API and
 * our internal representation of an error
 */
pub fn to_bot_error(error: lol::models::LolApiError) -> discord_bot_types::BotError {

    let error_code: u64 = match error.http_code.as_str() {
        // Too many requests too quick
        "429" => 429,
        "404" => 404,

        // Anything else is probably our bug
        x => 500
    };

    discord_bot_types::BotError {
        statusCode: error_code,
        body: error.description
    }
}