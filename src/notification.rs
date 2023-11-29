use rayon::prelude::*;
use std::collections::HashMap;
use std::error::Error;

use crate::args::Webhook;
use crate::Result;

pub fn send_notification(webhooks: Vec<Webhook>, message: String) -> Result<String> {
    match webhooks
        .par_iter()
        .map(|webhook| match webhook {
            Webhook::Discord(url) => send_discord_message(&url.to_string_lossy(), message.clone()),
        })
        .collect::<std::result::Result<String, Box<dyn Error + Send + Sync>>>()
    {
        Ok(res) => Ok(res),
        Err(err) => Err(err),
    }
}

fn send_discord_message(
    url: &str,
    message: String,
) -> std::result::Result<String, Box<dyn Error + Send + Sync>> {
    let mut map = HashMap::new();
    map.insert("content", message.clone());

    // Create a HTTP client.
    let client = reqwest::blocking::Client::new();

    // Create a HTTP request.
    let request = client.post(url).json(&map).build().unwrap();

    // Send the HTTP request and wait for the response.
    let response = client.execute(request)?.text()?;

    Ok(response)
}
