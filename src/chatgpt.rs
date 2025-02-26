use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use crate::event::Event;
use async_trait::async_trait;
use regex::Regex;
use tokio::sync::mpsc::UnboundedSender;

use crate::config::ChatGPTConfig;
use crate::llm::{LLMAnswer, LLMRole, LLM};
use reqwest::header::HeaderMap;
use serde_json::{json, Value};
use std;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ChatGPT {
    client: reqwest::Client,
    openai_api_key: String,
    model: String,
    url: String,
    messages: Vec<HashMap<String, String>>,
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
You need to define one whether in the configuration file or as an environment variable"#
                    );

                    std::process::exit(1);
                })
                .unwrap(),
        };

        Self {
            client: reqwest::Client::new(),
            openai_api_key,
            model: config.model,
            url: config.url,
            messages: Vec::new(),
        }
    }
}

#[async_trait]
impl LLM for ChatGPT {
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
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.openai_api_key).parse()?,
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

        messages.extend(self.messages.clone());

        let body: Value = json!({
            "model": self.model,
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

                            let msg = answer["choices"][0]["delta"]["content"]
                                .as_str()
                                .unwrap_or("\n");

                            if msg != "null" {
                                sender.send(Event::LLMEvent(LLMAnswer::Answer(msg.to_string())))?;
                            }

                            sleep(Duration::from_millis(1)).await;
                        }
                    }
                }
            }
            Err(e) => return Err(Box::new(e)),
        }

        Ok(())
    }
}
