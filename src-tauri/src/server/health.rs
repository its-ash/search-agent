use std::time::Duration;

pub async fn check(endpoint: &str) -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build();
    let Ok(client) = client else { return false };

    let health_url = format!("{endpoint}/health");
    if let Ok(resp) = client.get(&health_url).send().await {
        if resp.status().is_success() {
            return true;
        }
    }

    let model_url = format!("{endpoint}/v1/models");
    if let Ok(resp) = client.get(&model_url).send().await {
        if resp.status().is_success() {
            return true;
        }
    }

    let ollama_tags = format!("{endpoint}/api/tags");
    if let Ok(resp) = client.get(&ollama_tags).send().await {
        if resp.status().is_success() {
            return true;
        }
    }

    false
}
