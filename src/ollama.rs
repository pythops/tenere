use std::sync::atomic::{AtomicBool, Ordering};

use std::sync::Arc;

use crate::config::OllamaConfig;
use crate::event::Event;
use async_trait::async_trait;
use tokio::sync::mpsc::UnboundedSender;

use crate::llm::{LLMAnswer, LLMRole, LLM};
use reqwest::header::HeaderMap;
use serde_json::{json, Value};
use std;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Ollama {
    client: reqwest::Client,
    url: String,
    model: String,
    messages: Vec<HashMap<String, String>>,
}

impl Ollama {
    pub fn new(config: OllamaConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: config.url,
            model: config.model,
            messages: Vec::new(),
        }
    }
}

#[async_trait]
impl LLM for Ollama {
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
            "model": self.model,
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
                while let Some(chunk) = res.chunk().await? {
                    if terminate_response_signal.load(Ordering::Relaxed) {
                        sender.send(Event::LLMEvent(LLMAnswer::EndAnswer))?;
                        return Ok(());
                    }

                    let answer: Value = serde_json::from_slice(chunk.as_ref())?;

                    if answer["done"].as_bool().unwrap() {
                        sender.send(Event::LLMEvent(LLMAnswer::EndAnswer))?;
                        return Ok(());
                    }

                    let msg = answer["message"]["content"].as_str().unwrap_or("\n");

                    sender.send(Event::LLMEvent(LLMAnswer::Answer(msg.to_string())))?;
                }
            }
            Err(e) => return Err(Box::new(e)),
        }

        sender.send(Event::LLMEvent(LLMAnswer::EndAnswer))?;

        Ok(())
    }
}
