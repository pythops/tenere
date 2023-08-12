use crate::event::Event;
use regex::Regex;
use std::{thread, time};

use crate::config::ChatGPTConfig;
use crate::llm::{LLMAnswer, LLM};
use reqwest::header::HeaderMap;
use serde_json::{json, Value};
use std;
use std::collections::HashMap;
use std::io::Read;
use std::sync::mpsc::Sender;

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
        sender: &Sender<Event>,
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

        messages.extend(chat_messages);

        let body: Value = json!({
            "model": "gpt-3.5-turbo",
            "messages": messages,
            "stream": true,
        });

        let mut buffer = String::new();

        let response = self
            .client
            .post(&self.url)
            .headers(headers)
            .json(&body)
            .send()?;

        match response.error_for_status() {
            Ok(mut res) => {
                let _answser = res.read_to_string(&mut buffer)?;

                let re = Regex::new(r"data:\s(.*)").unwrap();

                sender.send(Event::LLMEvent(LLMAnswer::StartAnswer))?;

                for captures in re.captures_iter(&buffer) {
                    if let Some(data_json) = captures.get(1) {
                        if data_json.as_str() == "[DONE]" {
                            sender.send(Event::LLMEvent(LLMAnswer::EndAnswer)).unwrap();
                            break;
                        }
                        let x: Value = serde_json::from_str(data_json.as_str()).unwrap();

                        let msg = x["choices"][0]["delta"]["content"].as_str().unwrap_or("\n");

                        if msg != "null" {
                            sender
                                .send(Event::LLMEvent(LLMAnswer::Answer(msg.to_string())))
                                .unwrap();
                        }
                        thread::sleep(time::Duration::from_millis(100));
                    }
                }
            }
            Err(e) => return Err(Box::new(e)),
        }

        Ok(())
    }
}
