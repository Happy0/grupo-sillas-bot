use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{json, Value};

mod auth;
mod discord_bot_types;
mod lol;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: LambdaEvent<Value>) -> Result<Value, serde_json::Error> {
    let result = process_request(event).await;

    // AWS Lambda expects the returned 'body' field to be a JSON string, so we convert the bot response to a JSON string
    // and return it with the response headers and HTTP status code
    let body_with_json_string = result
        .map(|x| serde_json::to_string(&x.body)
            .map(|body_as_json_string| discord_bot_types::LambdaBotResponse {
                headers: x.headers,
                statusCode: x.statusCode,
                body: body_as_json_string
        }))
        .map(|x| x.map_err(|y| discord_bot_types::BotError {statusCode: 500, body: "Error marshalling to JSON".to_string()}))
        .and_then(|x| x);

    let send = match body_with_json_string {
        Err(bot_error) => serde_json::to_value(&bot_error),
        Ok(success) => serde_json::to_value(&success)
    };

    println!("Responding with...");
    println!("{:?}", send);

    return send;
}

async fn process_request(event: LambdaEvent<Value>) -> Result<discord_bot_types::BotResponse, discord_bot_types::BotError> {
    let (event, _context) = event.into_parts();
    auth::verify_request(&event).then(|| true).ok_or(discord_bot_types::BotError{statusCode: 401, body: "invalid request signature".to_string()})?;

    let event_body = event["body"].as_str();

    println!("Received: {:?}", event_body);

    let payload = event_body.ok_or(make_validation_error_response("Missing body".to_string()))?;
    let payload_value: discord_bot_types::DiscordReceivedCommand = 
        serde_json::from_str(payload)
            .map_err(|x| make_validation_error_response("Payload is not of expected DiscordReceivedCommand structure".to_string()))?;
    
    match (payload_value.typeField) {
        1 => {return Ok(make_ping_response())},
        2 => {
            let command = payload_value.data.ok_or(make_validation_error_response("Command missing 'data' field.".to_string()))?;
            return create_command_response(command).await;
        },
        _ => {
            return Err(make_error_response(400, "Unrecognised command type")); 
        }
    };

}

async fn create_command_response(command_data: discord_bot_types::Command ) -> Result<discord_bot_types::BotResponse, discord_bot_types::BotError> {

    let bot_response = match command_data.name.as_str() {
        "played" => {
            "aahh, i dunno yet."
        },
        x => {format!("Unrecognise command: {}", x)}
    };

    return Ok(discord_bot_types::BotResponse {
            headers: discord_bot_types::Headers {
                contentType: "application/json".to_string()
            },
            statusCode: 200,
            body: discord_bot_types::Body {
                typeField: 4,
                data: Some(discord_bot_types::Data {
                    tts: false,
                    content: bot_response.to_string()
                })
            }
    });
}

fn make_validation_error_response(error: String) -> discord_bot_types::BotError {
    return discord_bot_types::BotError {
        statusCode: 400,
        body: error
    } 
}

fn make_error_response(error_code: u64, description: &str) -> discord_bot_types::BotError {
    return discord_bot_types::BotError{
        statusCode: error_code,
        body: description.to_string()
    };
}

fn make_ping_response() -> discord_bot_types::BotResponse {
    return discord_bot_types::BotResponse {
        headers: discord_bot_types::Headers {
            contentType: "application/json".to_string()
        },
        statusCode: 200,
        body: discord_bot_types::Body {
            typeField: 1,
            data: None
        }
    }
}
