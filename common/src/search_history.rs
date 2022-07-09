use aws_sdk_dynamodb::{Client, Error};
use aws_sdk_dynamodb::model::{AttributeValue};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct SearchedDetails {
    pub discord_id: String,
    pub searched_name: String,
    pub times: u64,
    pub last_search: u64
}

pub async fn store_search(client: &Client, discord_user_id: &str, searched_for: &str) -> Result<(), Error> {
    let table = "grupoSillasBotTable";

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        // TODO: remove panic
        .expect("Time went backwards");

    let seconds_since_epoch = since_the_epoch.as_secs();

    let mut key_map: HashMap<std::string::String, AttributeValue> = HashMap::new();
    key_map.insert("partitionKey".to_string(), AttributeValue::S(discord_user_id.to_string()));
    key_map.insert("sortKey".to_string(), AttributeValue::S(searched_for.to_string()));

    let mut attribute_map: HashMap<std::string::String, AttributeValue> = HashMap::new();
    attribute_map.insert(":initial".to_string(), AttributeValue::N("1".to_string()));
    attribute_map.insert(":num".to_string(), AttributeValue::N("1".to_string()));
    attribute_map.insert(":now".to_string(), AttributeValue::N(seconds_since_epoch.to_string()));

    client
        .update_item()
        .table_name(table)
        .set_key(
            Some(key_map)
        )
        .update_expression("SET times = if_not_exists(times, :initial) + :num, last_search= :now")
        .set_expression_attribute_values(Some(attribute_map))
        .send()
        .await?;
   
    // TODO use UpdateItemOutput to return the new value for the row?
    return Ok(());
}

pub async fn get_searches(client: &Client, discord_user_id: &str) -> Result<Vec<SearchedDetails>, Error> {
    let table_name = "grupoSillasBotTable";

    let discord_attribute_id = AttributeValue::S(discord_user_id.to_string());
    let result = client
        .query()
        .table_name(table_name)
        .key_condition_expression("#partitionKey = :discord_id")
        .expression_attribute_names("#partitionKey", "partitionKey")
        .expression_attribute_values(
            ":discord_id",
            discord_attribute_id,
        )
        .send()
        .await?;

    let items = result.items;

    match items {
        None => {
            println!("None?");
            return Ok(Vec::new());
        },
        Some(results) => {
            println!("{:?}", results);

            let result: Vec<SearchedDetails> = results.into_iter().map(|x| get_searched_details(&x)).flatten().collect();
            return Ok(result);
        }
    }
}

fn get_searched_details(map: &HashMap<String, AttributeValue>) -> Option<SearchedDetails> {

    let discord_id = map.get("partitionKey").and_then(|x| x.as_s().ok() )?;
    let username = map.get("sortKey").and_then(|x| x.as_s().ok())?;
    let num_searches: u64 = map.get("times").and_then(|x| x.as_n().ok().and_then(|y| y.parse::<u64>().ok()))?;
    let last_search = map.get("last_search").and_then(|x| x.as_n().ok()).and_then(|y| y.parse::<u64>().ok())?;

    return Some(SearchedDetails {
        discord_id: discord_id.to_string(),
        searched_name: username.to_string(),
        times: num_searches,
        last_search: last_search
    })
}