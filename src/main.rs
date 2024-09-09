use anyhow::{Context, Result};
use env_logger::Env;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use structopt::StructOpt;
use url::Url;
use log::{info, error, debug};
use std::collections::HashMap;

#[derive(StructOpt)]
struct Cli {
    search_term: String,
}

#[derive(Deserialize, Debug)]
struct FeatureResponse {
    #[serde(rename = "featureIds")]
    feature_ids: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum BrowserSupport {
    Bool(bool),
    String(String),
    Object(HashMap<String, Value>),
}

#[derive(Deserialize, Serialize, Debug)]
struct FeatureData {
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    spec: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    mdn_url: String,
    #[serde(default)]
    support: HashMap<String, BrowserSupport>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Deserialize, Serialize)]
struct Stats {
    chrome: serde_json::Value,
    safari: serde_json::Value,
    firefox: serde_json::Value,
    edge: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let args = Cli::from_args();
    let client = Client::new();

    println!("Search term: {}", args.search_term);

    let feature_ids = get_feature_ids(&client, &args.search_term).await?;
    println!("Selected feature IDs: {:?}", feature_ids);

    let feature_data = get_feature_data(&client, &feature_ids).await?;

    println!("Feature data:");
    for (index, feature) in feature_data.iter().enumerate() {
        println!("Feature {}:", index + 1);
        println!("  Title: {}", feature.title);
        println!("  Description: {}", feature.description);
        println!("  Spec: {}", feature.spec);
        println!("  MDN URL: {}", feature.mdn_url);
        println!("  Support:");
        for (browser, support) in &feature.support {
            match support {
                BrowserSupport::Bool(b) => println!("    {}: {}", browser, b),
                BrowserSupport::String(s) => println!("    {}: {}", browser, s),
                BrowserSupport::Object(obj) => {
                    println!("    {}:", browser);
                    for (key, value) in obj {
                        println!("      {}: {}", key, value);
                    }
                }
            }
        }
        println!("  Extra information:");
        for (key, value) in &feature.extra {
            println!("    {}: {}", key, value);
        }
        println!();
    }

    Ok(())
}


async fn get_feature_ids(client: &Client, search_term: &str) -> Result<Vec<String>> {
    info!("Searching for feature IDs for term: '{}'", search_term);

    let mut base_url = Url::parse("https://caniuse.com/process/query.php")?;
    base_url.query_pairs_mut().append_pair("search", search_term);

    let url = base_url.to_string();

    debug!("Requesting URL: {}", url);

    let response = client.get(&url).send().await?;
    let status = response.status();
    let body = response.text().await?;

    info!("Response status: {}", status);
    debug!("Response body: {}", body);

    if !status.is_success() {
        error!("API request failed with status: {}", status);
        anyhow::bail!("API request failed with status: {}", status);
    }

    let parsed: FeatureResponse = serde_json::from_str(&body)
        .context("Failed to parse feature IDs response")?;

    debug!("Parsed response: {:?}", parsed);

    if parsed.feature_ids.is_empty() {
        anyhow::bail!("No feature IDs found for the given search term");
    }

    Ok(parsed.feature_ids.into_iter().take(2).collect())
}

async fn get_feature_data(client: &Client, feature_ids: &[String]) -> Result<Vec<FeatureData>> {
    let mut feature_data = Vec::new();

    for feature_id in feature_ids {
        info!("Fetching data for feature ID: {}", feature_id);

        let url = format!(
            "https://caniuse.com/process/get_feat_data.php?type=support-data&feat={}",
            feature_id
        );
        debug!("Requesting URL: {}", url);

        let response: Value = client
            .get(&url)
            .send()
            .await?
            .json()
            .await
            .context("Failed to parse feature data response")?;

        debug!("Received response for feature ID {}: {:?}", feature_id, response);

        // Parse the feature data
        if let Some(data) = response.as_array().and_then(|arr| arr.first()) {
            let feature: FeatureData = serde_json::from_value(data.clone())
                .context("Failed to parse feature data")?;
            feature_data.push(feature);
        }

        info!("Successfully parsed data for feature ID: {}", feature_id);
    }

    Ok(feature_data)
}
