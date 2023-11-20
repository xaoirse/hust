use std::collections::HashMap;

use crate::config::WebHook;
use crate::Result;

pub async fn send_notification(webhooks: Vec<WebHook>, message: String) -> Result<()> {
    for webhook in webhooks {
        match webhook {
            WebHook::Discord(webhook) => {
                send_discord_message(webhook, message.clone()).await?;
            }
        }
    }
    Ok(())
}

async fn send_discord_message(webhook: String, message: String) -> Result<String> {
    let mut map = HashMap::new();
    map.insert("content", message.clone());

    // Create a HTTP client.
    let client = reqwest::Client::new();

    // Create a HTTP request.
    let request = client.post(webhook).json(&map).build().unwrap();

    // Send the HTTP request and wait for the response.
    let response = client.execute(request).await?.text().await?;

    Ok(response)
}
