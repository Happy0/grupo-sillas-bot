use serde::{Deserialize, Serialize};

use lol;

pub struct Toolbox {
    pub lol_api_fetcher: lol::api_fetcher::BoundedHttpFetcher
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BotError {
    pub statusCode: u64,
    pub body: String
}

/**
 * Converts between an error received from the LoL API and
 * our internal representation of an error
 */
impl From<lol::models::LolApiError> for BotError {
    fn from(error: lol::models::LolApiError) -> Self {

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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Headers {
    #[serde(rename(serialize = "Content-Type", deserialize = "Content-Type"))]
    pub contentType: String
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub tts: bool,
    pub content: String
}

#[derive(Serialize, Deserialize)]
pub struct Body {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub typeField: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Data>
}

#[derive(Serialize, Deserialize)]
pub struct BotResponse {
    pub headers: Headers,
    pub statusCode: u64,
    pub body: Body
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LambdaBotResponse {
    pub headers: Headers,
    pub statusCode: u64,
    pub body: String
}

#[derive(Serialize, Deserialize)]
pub struct DiscordReceivedCommand {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub typeField: u64,
    pub token: String,
    pub data: Option<Command>
}

#[derive(Serialize, Deserialize)]
pub struct StringCommandOption {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    typeField: u64,
    pub name: String,
    pub value: String 
}

#[derive(Serialize, Deserialize)]
pub struct NumberCommandOption {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    typeField: u64,
    pub name: String,
    pub value: u64 
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum CommandOption {
    NumberCommandOption(NumberCommandOption),
    StringCommandOption(StringCommandOption)
}

#[derive(Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub name: String,
    pub options: Vec<CommandOption>
}

pub struct PlayedCommand {
    pub player_name: String,
    pub days: u64
}