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

async fn func(event: LambdaEvent<Value>) -> Result<Result<discord_bot_types::BotResponse, discord_bot_types::BotError>, Error> {
    let result = process_request(event).await;

    println!("Responding with...");
    println!("{}", serde_json::to_string(&result)?);

    return Ok(result);
}

async fn process_request(event: LambdaEvent<Value>) -> Result<discord_bot_types::BotResponse, discord_bot_types::BotError> {
    let (event, _context) = event.into_parts();
    auth::verify_request(&event).then(|| true).ok_or(discord_bot_types::BotError{statusCode: 401, body: "invalid request signature".to_string()})?;

    let event_body = event["body"].as_str();
    let payload = event_body.ok_or(make_validation_error_response("Missing body".to_string()))?;
    let payload: Value = serde_json::from_str(payload).map_err(|x| make_validation_error_response("Payload is not JSON object".to_string()))?;
    let command_data = &payload["data"];
    let interaction_type = payload["type"].as_i64().ok_or(make_validation_error_response("Missing type field".to_string()))?;

    match interaction_type {
        1 => {return Ok(make_ping_response())},
        2 => {
            return create_command_response(command_data);
        },
        _ => {
            return Err(make_error_response(400, "Unrecognised command type")); 
        }
    };

}

fn create_command_response(command_data: &Value) -> Result<discord_bot_types::BotResponse, discord_bot_types::BotError> {
    return Ok(discord_bot_types::BotResponse {
            headers: discord_bot_types::Headers {
                contentType: "application/json".to_string()
            },
            statusCode: 200,
            body: discord_bot_types::Body {
                typeField: 4,
                data: Some(discord_bot_types::Data {
                    tts: false,
                    content: "Congrats on ur command".to_string()
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
