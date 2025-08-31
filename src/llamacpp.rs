use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::event::Event;
use async_trait::async_trait;
use regex::Regex;
use tokio::sync::mpsc::UnboundedSender;

use crate::config::LLamacppConfig;
use crate::llm::{LLMAnswer, LLMRole, LLM};
use reqwest::header::HeaderMap;
use serde_json::{json, Value};
use std;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct LLamacpp {
    client: reqwest::Client,
    url: String,
    api_key: Option<String>,
    messages: Vec<HashMap<String, String>>,
}

impl LLamacpp {
    pub fn new(config: LLamacppConfig) -> Self {
        let api_key = match std::env::var("LLAMACPP_API_KEY") {
            Ok(key) => Some(key),
            Err(_) => config.api_key.clone(),
        };

        Self {
            client: reqwest::Client::new(),
            url: config.url,
            api_key,
            messages: Vec::new(),
        }
    }
}

#[async_trait]
impl LLM for LLamacpp {
    fn clear(&mut self) {
        self.messages = Vec::new();
    }

    fn append_chat_msg(&mut self, msg: String, role: LLMRole) {
        let mut conv: HashMap<String, String> = HashMap::new();
        conv.insert("role".to_string(), role.to_string());
        conv.insert("content".to_string(), msg);
        self.messages.push(conv);
    }

    async fn ask(
        &self,
        sender: UnboundedSender<Event>,
        terminate_response_signal: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse()?);

        if let Some(api_key) = &self.api_key {
            headers.insert("Authorization", format!("Bearer {}", api_key).parse()?);
        }

        let mut messages: Vec<HashMap<String, String>> = vec![
            (HashMap::from([
                ("role".to_string(), "system".to_string()),
                (
                    "content".to_string(),
                    "You are a helpful assistant.".to_string(),
                ),
            ])),
        ];

        messages.extend(self.messages.clone());

        let body: Value = json!({
            "messages": messages,
            "stream": true,
        });

        let response = self
            .client
            .post(&self.url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        match response.error_for_status() {
            Ok(mut res) => {
                sender.send(Event::LLMEvent(LLMAnswer::StartAnswer))?;
                let re = Regex::new(r"data:\s(.*)")?;
                while let Some(chunk) = res.chunk().await? {
                    let chunk = std::str::from_utf8(&chunk)?;

                    for captures in re.captures_iter(chunk) {
                        if let Some(data_json) = captures.get(1) {
                            if terminate_response_signal.load(Ordering::Relaxed) {
                                sender.send(Event::LLMEvent(LLMAnswer::EndAnswer))?;
                                return Ok(());
                            }

                            if data_json.as_str() == "[DONE]" {
                                sender.send(Event::LLMEvent(LLMAnswer::EndAnswer))?;
                                return Ok(());
                            }
                            let answer: Value = serde_json::from_str(data_json.as_str())?;

                            let msg = answer["choices"][0]["delta"]["content"].as_str();

                            if let Some(msg) = msg {
                                sender.send(Event::LLMEvent(LLMAnswer::Answer(msg.to_string())))?;
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(Box::new(e)),
        }

        sender.send(Event::LLMEvent(LLMAnswer::EndAnswer))?;

        Ok(())
    }
}
