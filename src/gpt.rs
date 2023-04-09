use reqwest::header::HeaderMap;
use serde_json::{json, Value};
use std;

#[derive(Debug)]
pub struct GPT {
    client: reqwest::Client,
}

impl Default for GPT {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl GPT {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn ask(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let url = "https://api.openai.com/v1/chat/completions";
        let token =
            std::env::var("OPENAI_API_KEY").expect("Can not find the OPENAI_API_KEY env variable");

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );

        let body: Value = json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {"role": "system", "content": "You are a helpful assistant."},
                {"role": "user", "content": prompt},
            ]
        });

        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        let response_body: Value = response.json().await?;

        let answer = response_body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap()
            .trim_matches('"')
            .to_string();

        Ok(answer)
    }
}
