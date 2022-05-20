use serde::{Deserialize, Serialize};

pub struct BotError {
    pub http_status: u64,
    pub description: String
}

#[derive(Serialize, Deserialize)]
pub struct Headers {
    #[serde(rename(serialize = "Content-Type", deserialize = "Content-Type"))]
    pub contentType: String
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    tts: bool,
    content: String
}

#[derive(Serialize, Deserialize)]
pub struct Body {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    typeField: u64,
    data: Option<Data>
}

#[derive(Serialize, Deserialize)]
pub struct BotResponse {
    pub headers: Headers,
    pub statusCode: u64,
    body: Body
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum CommandPrimitiveType {
    Integer(u64),
    String,
    Boolean
}

#[derive(Serialize, Deserialize)]
pub struct CommandOption {
    name: String,
    value: CommandPrimitiveType 
}

#[derive(Serialize, Deserialize)]
pub struct Command {
    id: String,
    name: String,
    options: Vec<CommandOption>
}