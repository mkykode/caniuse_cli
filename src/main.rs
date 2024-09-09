use anyhow::{Context, Result};
use env_logger::Env;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use structopt::StructOpt;
use url::Url;
use log::{info, error, debug};
use std::collections::HashMap;
use colored::*;
use tabled::{Table, Tabled};

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
    support: Option<HashMap<String, BrowserSupport>>,
    #[serde(default)]
    stats: Option<HashMap<String, HashMap<String, String>>>,
    #[serde(default)]
    notes_by_num: Option<HashMap<String, String>>, // New field
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Tabled)]
struct BrowserSupportRow {
    browser: String,
    support: String,
    notes: String,
}

fn get_support_emoji(support_value: &str) -> &str {
    match support_value {
        "false" => "❌",
        s if s.parse::<f32>().is_ok() => "✅",
        _ => match support_value.to_lowercase().as_str() {
            "y" | "true" => "✅",
            "n" | "false" => "❌",
            "a" | "partial" => "🟨",
            _ => "❓",
        },
    }
}

fn get_support_and_notes(support_info: &BrowserSupport, notes_by_num: &Option<HashMap<String, String>>) -> (String, String) {
    let (support_value_str, notes) = match support_info {
        BrowserSupport::Bool(b) => (b.to_string(), None),
        BrowserSupport::String(s) => (s.clone(), None),
        BrowserSupport::Object(obj) => {
            if let Some(version_added) = obj.get("version_added") {
                if let Some(version_str) = version_added.as_str() {
                    if version_str.contains('#') {
                        let notes = version_str
                            .split('#')
                            .skip(1)
                            .filter_map(|num| {
                                notes_by_num
                                    .as_ref()
                                    .and_then(|notes| notes.get(num).map(|note| format!("#{}: {}", num, note)))
                            })
                            .collect::<Vec<_>>()
                            .join("; ");
                        (version_str.to_string(), Some(notes))
                    } else {
                        (version_str.to_string(), None)
                    }
                } else {
                    (version_added.to_string(), None)
                }
            } else {
                ("unknown".to_string(), None)
            }
        }
    };

    let emoji = get_support_emoji(&support_value_str);
    let notes = notes.unwrap_or_else(|| String::new());

    (emoji.to_string(), notes)
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let args = Cli::from_args();
    let client = Client::new();

    println!("{} {}", "🔍".bold(), "Search term:".bold().green());
    println!("{}", args.search_term.yellow());

    let feature_ids = get_feature_ids(&client, &args.search_term).await?;
    println!("\n{} {}", "🏷️ ".bold(), "Selected feature IDs:".bold().green());
    for id in &feature_ids {
        println!("  • {}", id.yellow());
    }

    let feature_data = get_feature_data(&client, &feature_ids).await?;

    println!("\n{} {}", "📊".bold(), "Feature data:".bold().green());
    for (index, feature) in feature_data.iter().enumerate() {
        println!("\n{} {}", "🔹".bold(), format!("Feature {}:", index + 1).bold().blue());
        println!("  {} {}", "📌".bold(), format!("Title: {}", feature.title).bold());
        println!("  {} Description: {}", "📝".bold(), feature.description);
        println!("  {} Spec: {}", "📘".bold(), feature.spec);
        println!("  {} MDN URL: {}", "🔗".bold(), feature.mdn_url);

        println!("\n  {} {}", "🖥️ ".bold(), "Browser Compatibility:".bold());
        let mut support_data = Vec::new();

        if let Some(support) = &feature.support {
            for (browser, support_info) in support {
                let (emoji, notes) = get_support_and_notes(support_info, &feature.notes_by_num);
                let support_str = match support_info {
                    BrowserSupport::Bool(b) => b.to_string(),
                    BrowserSupport::String(s) => s.clone(),
                    BrowserSupport::Object(obj) => {
                        obj.get("version_added")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string()
                    },
                };

                support_data.push(BrowserSupportRow {
                    browser: format!("{} {}", emoji, browser),
                    support: support_str,
                    notes,
                });
            }
        } else if let Some(stats) = &feature.stats {
            for (browser, versions) in stats {
                let latest_version = versions
                    .keys()
                    .max_by(|a, b| {
                        a.parse::<f32>()
                            .unwrap_or(0.0)
                            .partial_cmp(&b.parse::<f32>().unwrap_or(0.0))
                            .unwrap()
                    })
                    .unwrap_or(&String::new())
                    .clone();
                let support_value = versions.get(&latest_version).unwrap_or(&String::new()).clone();
                let (emoji, notes) = get_support_and_notes(&BrowserSupport::String(support_value.clone()), &feature.notes_by_num);
                support_data.push(BrowserSupportRow {
                    browser: format!("{} {}", emoji, browser),
                    support: format!("{}: {}", latest_version, support_value),
                    notes,
                });
            }
        }

        if !support_data.is_empty() {
            let table = Table::new(support_data).to_string();
            println!("{}", table);
        } else {
            println!("  No compatibility data available.");
        }

        println!("\n  {} {}", "ℹ️ ".bold(), "Extra information:".bold());
        for (key, value) in &feature.extra {
            println!("    • {}: {}", key.bold(), value);
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

    Ok(parsed.feature_ids.into_iter().collect())
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
