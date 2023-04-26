use crate::config::ChatGPTConfig;
use crate::llm::LLM;
use reqwest::header::HeaderMap;
use serde_json::{json, Value};
use std;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ChatGPT {
    client: reqwest::blocking::Client,
    openai_api_key: String,
    url: String,
}

impl ChatGPT {
    pub fn new(config: ChatGPTConfig) -> Self {
        let openai_api_key = match std::env::var("OPENAI_API_KEY") {
            Ok(key) => key,
            Err(_) => config
                .openai_api_key
                .ok_or_else(|| {
                    eprintln!(
                        r#"Can not find the openai api key
You need to define one wether in the configuration file or as an environment variable"#
                    );

                    std::process::exit(1);
                })
                .unwrap(),
        };

        Self {
            client: reqwest::blocking::Client::new(),
            openai_api_key,
            url: config.url,
        }
    }
}

impl LLM for ChatGPT {
    fn ask(
        &self,
        chat_messages: Vec<HashMap<String, String>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.openai_api_key).parse().unwrap(),
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
            .post(&self.url)
            .headers(headers)
            .json(&body)
            .send()?;

        match response.error_for_status() {
            Ok(res) => {
                let response_body: Value = res.json()?;
                let answer = response_body["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap()
                    .trim_matches('"')
                    .to_string();

                Ok(answer)
            }
            Err(e) => Err(Box::new(e)),
        }
    }
}
