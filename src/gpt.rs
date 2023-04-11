use crate::app::AppResult;
use reqwest::header::HeaderMap;
use serde_json::{json, Value};
use std;
use std::collections::HashMap;

#[derive(Clone, Debug)]
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

    pub async fn ask(
        &self,
        chat_messages: Vec<HashMap<String, String>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = "https://api.openai.com/v1/chat/completions";
        let token = std::env::var("OPENAI_API_KEY")
            .expect("OPENAI_API_KEY environment variable is not set");

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );

        let mut messages: Vec<HashMap<String, String>> = vec![
            (HashMap::from([
                ("role".to_string(), "system".to_string()),
                (
                    "content".to_string(),
                    "You are a helpful assistant.".to_string(),
                ),
            ])),
        ];

        messages.extend(chat_messages);

        let body: Value = json!({
            "model": "gpt-3.5-turbo",
            "messages": messages
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
