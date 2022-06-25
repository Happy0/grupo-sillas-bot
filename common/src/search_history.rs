use aws_sdk_dynamodb::{Client, Error};
use aws_sdk_dynamodb::model::{AttributeValue};
use std::collections::HashMap;
use chrono;

pub struct SearchedDetails {
    discord_id: String,
    searched_name: String,
    times: u64,
    last_search: chrono::DateTime<chrono::FixedOffset>
}

pub async fn store_search(client: &Client, discord_user_id: &str, searched_for: &str) -> Result<SearchedDetails, Error> {
    let table = "grupoSillasBotTable";
    panic!("aaaahhhhh!!!")
}

pub async fn get_searches(client: &Client, discord_user_id: &str) -> Result<Option<SearchedDetails>, Error> {
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

    panic!("dfoijhsdiopjf");
}

fn get_searched_details(map: HashMap<String, AttributeValue>) -> Option<SearchedDetails> {

    let discord_id = map.get("partitionKey").and_then(|x| x.as_s().ok() )?;
    let username = map.get("sortKey").and_then(|x| x.as_s().ok())?;
    let num_searches: u64 = map.get("times").and_then(|x| x.as_n().ok().and_then(|y| y.parse::<u64>().ok()))?;
    let last_search = map.get("lastSearch").and_then(|x| x.as_s().ok()).and_then(|y| chrono::DateTime::parse_from_rfc2822(y).ok()  )?;

    return Some(SearchedDetails {
        discord_id: discord_id.to_string(),
        searched_name: username.to_string(),
        times: num_searches,
        last_search: last_search
    })
}