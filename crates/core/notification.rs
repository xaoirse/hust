use std::collections::HashMap;
use std::error::Error;

use rayon::prelude::*;

use crate::config::WebHook;
use crate::Result;

pub fn send_notification(webhooks: Vec<WebHook>, message: String) -> Result<String> {
    match webhooks
        .par_iter()
        .map(|webhook| match webhook {
            WebHook::Discord(webhook) => send_discord_message(webhook, message.clone()),
        })
        .collect::<std::result::Result<String, Box<dyn Error + Send + Sync>>>()
    {
        Ok(res) => Ok(res),
        Err(err) => Err(err),
    }
}

fn send_discord_message(
    webhook: &str,
    message: String,
) -> std::result::Result<String, Box<dyn Error + Send + Sync>> {
    let mut map = HashMap::new();
    map.insert("content", message.clone());

    // Create a HTTP client.
    let client = reqwest::blocking::Client::new();

    // Create a HTTP request.
    let request = client.post(webhook).json(&map).build().unwrap();

    // Send the HTTP request and wait for the response.
    let response = client.execute(request)?.text()?;

    Ok(response)
}
