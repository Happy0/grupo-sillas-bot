// TODO: helper function for doing fetches in rate limit friendly way
// Returns a future, internally processes sequentially...
use crate::lol::models;

use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender, Receiver};
use reqwest::Response;
use reqwest:: Error;
use reqwest::Client;
use tokio::task;
use tokio::sync::oneshot;
use std::time::{Duration, SystemTime};
use futures::future::FutureExt;
use tokio::time::sleep;
use core::clone::Clone;

pub struct SendCommand {
    url: String,
    receiver: oneshot::Sender<Result<Response, Error>>
}

pub struct BoundedHttpFetcher {
    sender: Sender<SendCommand>,
    per_second_limit: u64,
    per_minute_limit: u64
}

pub async fn create_lol_client(per_second_limit: u64, per_minute_limit: u64) -> BoundedHttpFetcher {
    let (tx, rx): (Sender<SendCommand>, Receiver<SendCommand>) = mpsc::channel(32);

    let fetcher = BoundedHttpFetcher {
        sender: tx,
        per_second_limit: per_second_limit,
        per_minute_limit: per_minute_limit
    };

    task::spawn(handle_requests(rx, per_second_limit, per_minute_limit));

    return fetcher;
}

pub async fn get_request(fetcher: &BoundedHttpFetcher, url: String) -> Result<Response, models::LolApiError> {
    let (tx, rx) = oneshot::channel();

    let command = SendCommand {
        url: url,
        receiver: tx
    };

    fetcher.sender.send(command).await.map_err(|err| models::LolApiError {
        description: format!("Could not send HTTP request, err: {}.", err),
        http_code: "500".to_string()
    })?;

    let x = rx.await.map_err(|err| models::LolApiError {
        description: format!("Could not await HTTP response, err: {}.", err ),
        http_code: "500".to_string()
    })?;
    
    return x.map_err(|x| models::LolApiError{
        description: format!("HTTP response error, status: {}, error{}", 
            x.status().map(|x| x.as_u16().to_string()).unwrap_or("None".to_string()), x
        ),
        http_code: "500".to_string()
    });
}

pub async fn handle_requests(mut receiver: Receiver<SendCommand>, per_second_limit: u64, per_minute_limit: u64) {

    let mut request_count: u64 = 0;
    let client = reqwest::Client::new();

    // A crude way of making sure we don't exceed the rate limit of the LoL API (at least the per second one for now)
    while let Some(cmd) = receiver.recv().await {
        let request_url = cmd.url;
        let sender = cmd.receiver;
        let cloned_client = client.clone();

        task::spawn (async move {

            let result = cloned_client
                .get(request_url)
                .send()
                .then(|res| async move {sender.send(res)})
                .await;

            if (!result.is_ok()) {
                println!("Ahh not")
            }
        });


        //let x = sender.send(result);

        // if !x.is_ok() {
        //     println!("Could not send result for {}", request_url);
        // }

        if request_count < per_second_limit {
            request_count = request_count + 1;
        } else {
            println!("Sleeping!");
            sleep(Duration::from_secs(1)).await;
            request_count = 0;
        }


    }
}