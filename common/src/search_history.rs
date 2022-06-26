use aws_sdk_dynamodb::{Client, Error};
use aws_sdk_dynamodb::model::{AttributeValue};
use std::collections::HashMap;

pub struct SearchedDetails {
    pub discord_id: String,
    pub searched_name: String,
    pub times: u64,
    pub last_search: u64
}

pub async fn store_search(client: &Client, discord_user_id: &str, searched_for: &str) -> Result<SearchedDetails, Error> {
    let table = "grupoSillasBotTable";

    // todo: fill these in:

    let key_map: HashMap<std::string::String, AttributeValue> = HashMap::new();
    let attribute_map: HashMap<std::string::String, AttributeValue> = HashMap::new();

    let request = client
        .update_item()
        .table_name(table)
        .set_key(
            Some(key_map)
        )
        .update_expression("SET visits = if_not_exists(times, :initial) + :num, last_search= :now")
        .set_expression_attribute_values(Some(attribute_map));
    
    panic!("aaaahhhhh!!!");
}

pub async fn get_searches(client: &Client, discord_user_id: &str) -> Result<Vec<SearchedDetails>, Error> {
    let table_name = "grupoSillasBotTable";

    let discord_attribute_id = AttributeValue::S(discord_user_id.to_string());
    let result = client
        .query()
        .table_name(table_name)
        .key_condition_expression("partitionKey = :discord_id")
        .expression_attribute_values(
            ":discord_id",
            discord_attribute_id,
        )
        .send()
        .await?;

    let items = result.items;

    match items {
        None => {
            return Ok(Vec::new());
        },
        Some(results) => {
            let result: Vec<SearchedDetails> = results.into_iter().map(|x| get_searched_details(&x)).flatten().collect();
            return Ok(result);
        }
    }
}

fn get_searched_details(map: &HashMap<String, AttributeValue>) -> Option<SearchedDetails> {

    let discord_id = map.get("partitionKey").and_then(|x| x.as_s().ok() )?;
    let username = map.get("sortKey").and_then(|x| x.as_s().ok())?;
    let num_searches: u64 = map.get("times").and_then(|x| x.as_n().ok().and_then(|y| y.parse::<u64>().ok()))?;
    let last_search = map.get("lastSearch").and_then(|x| x.as_n().ok()).and_then(|y| y.parse::<u64>().ok())?;

    return Some(SearchedDetails {
        discord_id: discord_id.to_string(),
        searched_name: username.to_string(),
        times: num_searches,
        last_search: last_search
    })
}