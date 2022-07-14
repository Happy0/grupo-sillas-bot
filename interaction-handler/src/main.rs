use common::discord_bot_types;
use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{Value, json};
use aws_sdk_sqs::Client;
use aws_sdk_dynamodb;
use aws_config::meta::region::RegionProviderChain;
use std::env;
use chrono;
use common;

mod auth;
mod lol_command;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let region_provider = RegionProviderChain::default_provider().or_else("eu-west-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);
    let dynamo_client = aws_sdk_dynamodb::Client::new(&config);

    let func = service_fn(|x| func(&client, &dynamo_client, x));
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(sqs_client: &Client, dynamo_client: &aws_sdk_dynamodb::Client, event: LambdaEvent<Value>) -> Result<Value, serde_json::Error> {
    let result = process_request(sqs_client, dynamo_client, event).await;

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

async fn process_request(sqs_client: &Client, dynamo_client: &aws_sdk_dynamodb::Client, event: LambdaEvent<Value>) -> Result<discord_bot_types::BotResponse, discord_bot_types::BotError> {
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
            let member = payload_value.member.ok_or(make_validation_error_response("Command missing 'member' field.".to_string()))?;

            let played_command = lol_command::build_played_command(command, member.user.id, payload_value.token, payload_value.application_id);

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
        4 => {
            let command = payload_value.data.ok_or(make_validation_error_response("Command missing 'data' field.".to_string()))?;
            let options = command.options;
            let discord_user_id = payload_value.member.map(|member| member.user.id);

            let suggestions = match discord_user_id {
                None => Vec::new(),
                Some(user_id) => {
                        generate_username_autocomplete_suggestions(dynamo_client, &user_id, options).await
                }
            };

            return Ok(discord_bot_types::BotResponse {
                headers: discord_bot_types::Headers {
                    contentType: "application/json".to_string()
                },
                statusCode: 200,
                body: discord_bot_types::Body {
                    typeField: 8,
                    data: Some(
                        discord_bot_types::Data {
                            tts: None,
                            content: None,
                            choices: Some(suggestions)
                        }
                    )
                }
            });
        }
        _ => {
            return Err(make_error_response(400, "Unrecognised command type")); 
        }
    };
}

async fn generate_username_autocomplete_suggestions(
    dynamo_client: &aws_sdk_dynamodb::Client,
    discord_user_id: &str,
    input: Vec<discord_bot_types::CommandOption>) -> Vec<discord_bot_types::StringChoice> {
    let name_field = input.into_iter().find_map(|x| match x {
        discord_bot_types::CommandOption::StringCommandOption(y) if y.name == "user" && y.focused == Some(true) => Some(y.value),
        _ => None
    });

    match name_field {
        None => return Vec::new(),
        Some(name_prefix) => {
            let searches = common::search_history::get_searches(dynamo_client, discord_user_id).await;

            match searches {
                Err(_) => {
                    return Vec::new()
                },
                Ok(res) => {
                    return res.into_iter().filter(|item| name_prefix.is_empty() || item.searched_name.starts_with(&name_prefix))
                        .map(|item| discord_bot_types::StringChoice {
                            name: "user".to_string(),
                            value: item.searched_name
                        }).collect();
                }
            }
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
    })?;

    println!("Queue URL is {}", queue_url);

    let now = chrono::offset::Utc::now();

    let mut dedup_id = format!("{}-{}-{}", played_command.player_name, played_command.days, now.to_rfc3339());
    dedup_id.retain(|c| !c.is_whitespace());

    let send_result = sqs_client
        .send_message()
        .queue_url(queue_url)
        .message_body(msg_body)
        .message_group_id("LolCommandGroup")
        .message_deduplication_id(dedup_id)
        .send()
        .await;

    println!("SQS send result: {:?}", send_result);

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
