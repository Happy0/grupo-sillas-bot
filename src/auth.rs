use ed25519_dalek::{PublicKey, Verifier, Signature};
use serde_json::{Value};
use std::env;
use hex;

/**
 * Verifies authorization via nacl
 */
pub fn verify_request(event: &Value) -> bool {
    let headers = &event["multiValueHeaders"];

    let public_key = env::var("DISCORD_BOT_PUBLIC_KEY");
    let pub_key_undefined = public_key.is_err();

    let signature = &headers["x-signature-ed25519"].as_array().and_then(|arr| arr.get(0).and_then(|x | x.as_str()));
    let timestamp = &headers["x-signature-timestamp"].as_array().and_then(|arr| arr.get(0).and_then(|x| x.as_str()));
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
            println!("Unexpected missing header / value while verifying request signature.");
            println!("public_key: {}, signature: {}, timestamp: {}, body: {}", pub_key_undefined, signature.is_none(), timestamp.is_none(), body.is_none());
            false
        }
    }
}