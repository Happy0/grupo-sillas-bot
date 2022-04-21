use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{json, Value};
use std::env;
use ed25519_dalek::{PublicKey, Verifier, Signature};
use hex;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {

    let (event, _context) = event.into_parts();

    let authorized_request = verify_request(event);

    if authorized_request {
        return Ok(json!(
            { 
                "statusCode": 200,
                "body": json!({
                    "type": 1
                }).to_string()
            })
        )
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

/**
 * Verifies authorization via nacl
 */
fn verify_request(event: Value) -> bool {
    let headers = &event["multiValueHeaders"];

    let public_key = env::var("PUBLIC_KEY");
    let signature = &headers["x-signature-ed25519"].as_str();
    let timestamp = &headers["x-signature-timestamp"].as_str();
    let body = &event["body"].as_str();

    // TODO: improve nesting using flat_map if I can be bothered
    match (public_key, signature, timestamp, body) {
        (Ok(pub_key), Some(sig), Some(ts), Some(body)) => {
            let public_key_bytes = hex::decode(pub_key);
            let signature_bytes = hex::decode(sig);

            match (public_key_bytes, signature_bytes) {
                (Ok(pub_bytes), Ok(sig_bytes)) => {
                    let x = PublicKey::from_bytes(&pub_bytes);
                    let y = Signature::from_bytes(&sig_bytes);

                    match (x,y) {
                        (Ok(key), Ok(sig)) => {
                            let together = format!("{}{}", ts, body);

                            let result = key.verify(together.as_bytes(), &sig);

                            return result.is_ok();
                        }
                        (_,_) => {
                            println!("Failed to construct pub key or sig from bytes");
                            return false;
                        }
                    };

                }
                _ => {
                    println!("Failed to parse pub_key or sig_bytes from hex to bytes");
                    return false
                }
            } 
        },
        _ => {
            println!("Unexpected missing header / value which verifying request signature.");
            false
        }
    }
}