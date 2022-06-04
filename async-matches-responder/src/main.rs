use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{json, Value};
use lol;
use common;
mod models;
mod lol_command;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let api_fetcher: lol::api_fetcher::BoundedHttpFetcher = lol::api_fetcher::create_lol_client(20,100);
    let client = reqwest::Client::new();
    let toolbox = models::Toolbox {
        lol_api_fetcher: api_fetcher,
        discord_http_client: client
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
                let result = handle_played_command(&toolbox.lol_api_fetcher, &toolbox.discord_http_client, &command).await;
                println!("Result: {:?}", result);
            }
        }
    }

    return Ok(json!({}));
}

async fn handle_played_command(
    lol_api_fetcher: &lol::api_fetcher::BoundedHttpFetcher,
    discord_http_client: &reqwest:: Client,
    command: &common::discord_bot_types::PlayedCommand) -> Result<(), common::discord_bot_types::BotError> {

    let result = lol_command::execute_played_command(lol_api_fetcher, command).await?;

    let discord_content_body = models::DiscordResponseBody {
        content: result
    };

    let body = serde_json::to_string(&discord_content_body).map_err(|x| common::discord_bot_types::BotError {
        statusCode: 500,
        body: "Could not write type to JSON string".to_string()
    })?;

    let mut headers = reqwest::header::HeaderMap::new();
    let value = reqwest::header::HeaderValue::from_static("application/json");
    let x = headers.insert("Content-Type", value);


    let request_url = format!("https://discord.com/api/webhooks/{}/{}/messages/@original", command.application_id, command.token);
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

    Ok(())
}