use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{json, Value};
use lol;
mod models;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let api_fetcher: lol::api_fetcher::BoundedHttpFetcher = lol::api_fetcher::create_lol_client(20,100);

    let toolbox = models::Toolbox {
        lol_api_fetcher: api_fetcher
    };

    let func = service_fn(|x| func(&toolbox, x));
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(toolbox: &models::Toolbox, lambda_event: LambdaEvent<Value>) -> Result<Value, serde_json::Error> {
    let (event, _context) = lambda_event.into_parts();

    println!("Received: {:?}", event);

    return Ok(json!({"stuff": "hi"}))
}