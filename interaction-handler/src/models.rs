use serde::{Deserialize, Serialize};
use lol;
use common::discord_bot_types::{BotError};

pub struct Toolbox {
    pub lol_api_fetcher: lol::api_fetcher::BoundedHttpFetcher
}

/**
 * Converts between an error received from the LoL API and
 * our internal representation of an error
 */
pub fn to_bot_error(error: lol::models::LolApiError) -> BotError {

    let error_code: u64 = match error.http_code.as_str() {
        // Too many requests too quick
        "429" => 429,
        "404" => 404,

        // Anything else is probably our bug
        x => 500
    };

    BotError {
        statusCode: error_code,
        body: error.description
    }
}

