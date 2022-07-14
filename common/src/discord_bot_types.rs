use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct BotError {
    pub statusCode: u64,
    pub body: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Headers {
    #[serde(rename(serialize = "Content-Type", deserialize = "Content-Type"))]
    pub contentType: String
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    // This is maybe a bit hacky. Could use a generic for 'data' field instead?
    pub tts: Option<bool>,
    pub content: Option<String>,
    pub choices: Option<Vec<StringChoice>>
}

#[derive(Serialize, Deserialize)]
pub struct Body {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub typeField: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Data>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StringChoice {
    pub name: String,
    pub value: String
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
    pub application_id: String,
    pub data: Option<Command>,
    pub member: Option<Member>
}

#[derive(Serialize, Deserialize)]
pub struct StringCommandOption {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub typeField: u64,
    pub name: String,
    pub value: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused: Option<bool>
}

#[derive(Serialize, Deserialize)]
pub struct NumberCommandOption {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    typeField: u64,
    pub name: String,
    pub value: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused: Option<bool>
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

#[derive(Serialize, Deserialize)]
pub struct Member {
    pub user: User
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String
}

#[derive(Serialize, Deserialize)]
pub struct PlayedCommand {
    pub token: String,
    pub application_id: String,
    pub discord_user_id: String,
    pub player_name: String,
    pub days: u64,
    pub game_type: Option<String>
}