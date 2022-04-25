use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{json, Value};

mod auth;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {

    let (event, _context) = event.into_parts();

    println!("{}",event.to_string());
    let event_body = event["body"].as_str();
    let authorized_request = auth::verify_request(&event);

    if authorized_request {

        match event_body {
            Some(x) => {
                return Ok(handle_request(x));
            }
            None => {
                return Ok(json!({
                    "statusCode": 400,
                    "body": "No body"
                }))
            }
        }

    } else {
        println!("Not authorized :o");
        return Ok(json!(
            { 
                "statusCode": 401,
                "body": "invalid request signature"
            })
        )
    }
}

fn handle_request(event_body: &str) -> Value {
    println!("body: {}", event_body);
    let payload: Result<Value, _> = serde_json::from_str(event_body);

    match payload {
        Ok(body) => {
            let interaction_type = body["type"].as_i64();
            let req_type = body["data"]["type"].as_i64();

            if (interaction_type == Some(1)) {
                let ping_response: Value = json!(
                    { 
                        "statusCode": 200,
                        "body": json!({
                            "type": 1
                        }).to_string()
                    });

                return ping_response;
            }

            println!("Trying to respond");
            return json!(
                { 
                    "statusCode": 200,
                    "body": {
                        "type": 4,
                        "data": {
                            "tts": false,
                            "content": "Congrats on sending your command!",
                            "embeds": [],
                            "allowed_mentions": { "parse": [] }
                        }
                    }
                });

        }
        Err(_) => {
            println!("Not a JSON payload");
            return 
                json!({
                    "statusCode": 400,
                    "body": "Not a JSON payload"
                })
            }
        
    }
}
    


