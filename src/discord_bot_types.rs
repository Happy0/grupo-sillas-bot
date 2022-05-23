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