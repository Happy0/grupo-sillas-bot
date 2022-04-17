use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {

    let (event, _context) = event.into_parts();

    println!("{}",event.to_string());

    Ok(json!(
        { 
            "statusCode": 200,
            "body": json!({
                "type": 1
            })
        })
    )
}