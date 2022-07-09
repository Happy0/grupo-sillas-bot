use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{json, Value};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{Client};

use lol;
use common;
mod models;
mod lol_command;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let region_provider = RegionProviderChain::default_provider().or_else("eu-west-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let dynamo_client = Client::new(&config);

    let api_fetcher: lol::api_fetcher::BoundedHttpFetcher = lol::api_fetcher::create_lol_client(20,100);
    let client = reqwest::Client::new();
    let toolbox = models::Toolbox {
        lol_api_fetcher: api_fetcher,
        discord_http_client: client,
        dynamo_client: dynamo_client
    };

    let func = service_fn(|x| func(&toolbox, x));
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(toolbox: &models::Toolbox, lambda_event: LambdaEvent<models::QueueBatch>) -> Result<Value, serde_json::Error> {
    let (event, _context) = lambda_event.into_parts();
    println!("Received: {:?}", event);

    for record in event.Records.iter() {
        let request: Result<common::discord_bot_types::PlayedCommand, _> = serde_json::from_str(&record.body);

        match request {
            Err(err) => println!("Could not parse played command to type"),
            Ok(command) => { 
                let result = handle_played_command(
                    &toolbox.lol_api_fetcher, &toolbox.discord_http_client,
                    &toolbox.dynamo_client,
                    &command
                ).await;
                println!("Result: {:?}", result);
            }
        }
    }

    return Ok(json!({}));
}

async fn handle_played_command(
    lol_api_fetcher: &lol::api_fetcher::BoundedHttpFetcher,
    discord_http_client: &reqwest:: Client,
    dynamo_client: &Client,
    command: &common::discord_bot_types::PlayedCommand) -> Result<(), common::discord_bot_types::BotError> {

    let result = lol_command::execute_played_command(lol_api_fetcher, command).await;

    let message = match &result {
        Err(bot_error) if bot_error.statusCode == 429 => "Too many requests in a short period of time, try again in a minute.".to_string(),
        Err(bot_error) if bot_error.statusCode == 404 => "User not found.".to_string(),
        Ok(msg) => msg.to_string(),
        Err(bot_error) => {
            println!("Unknown error: {:?}", bot_error);
            "An unknown error occurred.".to_string()
        }
    };

    let discord_content_body = models::DiscordResponseBody {
        content: message.to_string()
    };

    let body = serde_json::to_string(&discord_content_body).map_err(|x| common::discord_bot_types::BotError {
        statusCode: 500,
        body: "Could not write type to JSON string".to_string()
    })?;

    let response_future = send_response(discord_http_client, body, &command.application_id, &command.token);

    if result.is_ok() {
        let user_count_future = update_user_count(dynamo_client, &command.discord_user_id, &command.player_name);

        let (x, dynamo_result) = tokio::join!(response_future, user_count_future);

        println!("Dynamo update result: {:?}", dynamo_result);
    } else {
        response_future.await;
    }

    Ok(())
}

async fn send_response(discord_http_client: &reqwest:: Client, body: String, application_id: &str, token: &str) {
    let mut headers = reqwest::header::HeaderMap::new();
    let value = reqwest::header::HeaderValue::from_static("application/json");
    headers.insert("Content-Type", value);

    let request_url = format!("https://discord.com/api/webhooks/{}/{}/messages/@original", application_id, token);
    let send_result = discord_http_client
        .patch(request_url)
        .headers(headers)
        .body(body)
        .send()
        .await;

    println!("Discord send result: {:?}", send_result);

    let result_text = send_result.map( move |x| {x.text()} );

    match result_text {
        Ok(txt) => println!("{:?}", txt.await),
        Err(err) => println!("{:?}", err)
    }
}

async fn update_user_count(dynamo_client: &Client, discord_user_id: &str, searched_for: &str) -> Result<(), aws_sdk_dynamodb::Error> {
    let x = common::search_history::store_search(dynamo_client, discord_user_id, searched_for);

    return x.await;
}