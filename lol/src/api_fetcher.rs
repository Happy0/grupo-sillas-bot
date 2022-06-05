// TODO: helper function for doing fetches in rate limit friendly way
// Returns a future, internally processes sequentially...

use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender, Receiver};
use reqwest::Response;
use reqwest:: Error;
use tokio::task;
use tokio::sync::oneshot;
use std::time::{Duration};
use tokio::time::sleep;
use core::clone::Clone;
use crate::models;

pub struct SendCommand {
    url: String,
    receiver: oneshot::Sender<Result<Response, Error>>
}

pub struct BoundedHttpFetcher {
    sender: Sender<SendCommand>,
    per_second_limit: u64,
    per_minute_limit: u64
}

pub fn create_lol_client(per_second_limit: u64, per_minute_limit: u64) -> BoundedHttpFetcher {
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

    while let Some(cmd) = receiver.recv().await {
        let request_url = cmd.url;
        let sender = cmd.receiver;
        let cloned_client = client.clone();

        task::spawn (async move {
            println!("{}", request_url);
            let result = send_request(cloned_client, request_url).await;
            let send_result = sender.send(result);

            if !send_result.is_ok() {
                println!("Ahh not okie (could not send to oneshot channel)")
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
            sleep(Duration::from_secs(2)).await;
            request_count = 0;
        }


    }
}

async fn send_request(client: reqwest::Client, request_url: String) -> Result<reqwest::Response, reqwest::Error> {

    // TODO: Make this less branchy by creating a shared error result and using result type ? macro.
    let mut attempts_remaining: u64 = 5;
    loop {
        let result = client
            .get(&request_url)
            .send()
            .await;

        match &result {
            Err(x) => {
                return result;
            },
            Ok(http_result) => {
                if http_result.status() == 429 {
                    println!("It's a 429, waiting...");

                    let headers = http_result.headers();
                    let retry_after = &headers.get("Retry-After");

                    match retry_after {
                        Some(val) => {
                            let header_as_str = val.to_str();

                            match header_as_str {
                                Err(err) => {
                                    println!("Could not parse Retry-After header value to string.");
                                    return result;
                                },
                                Ok(header_str) => {
                                    let wait_for = header_str.parse::<u64>();

                                    match wait_for {
                                        Ok(wait_for) => {
                                            println!("Sleeping for {}", wait_for);
                                            sleep(Duration::from_secs(wait_for)).await;
                                        },
                                        Err(err) => {
                                            println!("Could not parse retry after header to int");
                                            return result;
                                        }
                                    }
                                }
                            }

                        },
                        None => {
                            println!("No Retry-After header");
                            return result;
                        }
                    }


                } else {
                    return result;
                }
            }
        }

        attempts_remaining = attempts_remaining - 1;

        if attempts_remaining <= 0 {
            return result;
        }
    }
}