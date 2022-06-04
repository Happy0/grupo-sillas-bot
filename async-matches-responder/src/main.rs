use lambda_runtime::{service_fn, LambdaEvent, Error};
use lol;
mod models;


#[tokio::main]
async fn main() -> Result<(), Error> {
    let api_fetcher: lol::api_fetcher::BoundedHttpFetcher = lol::api_fetcher::create_lol_client(20,100);

    let toolbox = models::Toolbox {
        lol_api_fetcher: api_fetcher
    };

    // let func = service_fn(|x| func(&toolbox, x));
    // lambda_runtime::run(func).await?;
    Ok(())
}