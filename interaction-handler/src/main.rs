use common::discord_bot_types;
use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{Value, json};
use aws_sdk_sqs::Client;
use aws_config::meta::region::RegionProviderChain;
use std::env;

mod auth;
mod lol_command;
mod models;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let region_provider = RegionProviderChain::default_provider().or_else("eu-west-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    let func = service_fn(|x| func(&client, x));
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(sqs_client: &Client, event: LambdaEvent<Value>) -> Result<Value, serde_json::Error> {
    let result = process_request(sqs_client, event).await;

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

async fn process_request(sqs_client: &Client, event: LambdaEvent<Value>) -> Result<discord_bot_types::BotResponse, discord_bot_types::BotError> {
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
            let played_command = lol_command::build_played_command(command, payload_value.token, payload_value.application_id);

            match played_command {
                Err(x) => {
                    return Err(make_error_response(400, "Could not parse options"))
                },
                Ok (options) => {
                    write_command_to_queue(sqs_client, options).await?;
                    return create_deferred_command_response()
                }
            }

        },
        _ => {
            return Err(make_error_response(400, "Unrecognised command type")); 
        }
    };
}

async fn write_command_to_queue(sqs_client: &Client, played_command: common::discord_bot_types::PlayedCommand) -> Result<(), discord_bot_types::BotError> {

    let queue_url = env::var("MATCHES_QUEUE_URL").map_err(|x| discord_bot_types::BotError {
        statusCode: 500,
        body: "Missing MATCHES_QUEUE_URL environment variable".to_string()
    })?;

    let msg_body = serde_json::to_string(&played_command).map_err(|X| discord_bot_types::BotError {
        statusCode: 500,
        body: "Could not write SQS payload to JSON string".to_string()
    });

    sqs_client
        .send_message()
        .queue_url(queue_url)
        .message_body("hello from my queue")
        .message_group_id("LolCommandGroup")
        .send()
        .await;

    Ok(())
}

fn create_deferred_command_response() -> Result<discord_bot_types::BotResponse, discord_bot_types::BotError> {

    return Ok(discord_bot_types::BotResponse {
            headers: discord_bot_types::Headers {
                contentType: "application/json".to_string()
            },
            statusCode: 200,
            body: discord_bot_types::Body {
                typeField: 5,
                data: None
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
