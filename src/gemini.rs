// gemini.rs
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use crate::event::Event;
use async_trait::async_trait;
use regex::Regex;
use tokio::sync::mpsc::UnboundedSender;
use crate::config::GeminiConfig;
use crate::llm::{LLMAnswer, LLMRole, LLM};
use reqwest::header::HeaderMap;
use serde_json::{json, Value};
use std;
use std::collections::HashMap;

// Configuration file example
//
// [gemini]
// base_url = "https://generativelanguage.googleapis.com/v1beta/models"
// model = "gemini-2.0-flash"
//

#[derive(Clone, Debug)]
pub struct Gemini {
    client: reqwest::Client,
    gemini_api_key: String,
    model: String,
    url: String,
    messages: Vec<HashMap<String, String>>,
}

impl Gemini {
    pub fn extract_text_from_gemini_response(json_response: &Value) -> Option<String> {
        json_response
            .get("candidates")?
            .as_array()?
            .first()?
            .get("content")?
            .get("parts")?
            .as_array()?
            .first()?
            .get("text")?
            .as_str()
            .map(String::from)
    }
    pub fn new(config: GeminiConfig) -> Self {
        let gemini_api_key = match std::env::var("GEMINI_API_KEY") {
            Ok(key) => key,
            Err(_) => config
                .gemini_api_key
                .ok_or_else(|| {
                    eprintln!(
                    r#"Can not find the gemini api key
You need to define one whether in the configuration file or as an environment variable"#
                );
                    std::process::exit(1);
                })
                .unwrap(),
        };

        let base_url = if config.url.is_empty() {
            GeminiConfig::default_url()
        } else {
            config.url.clone()
        };

        let fix_url = format!("{}/{}:generateContent", base_url, config.model);
        Self {
            client: reqwest::Client::new(),
            gemini_api_key,
            model: config.model,
            url: fix_url,
            messages: Vec::new(),
        }
    }
}

#[async_trait]
impl LLM for Gemini {
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
        // Check if the URL contains "openai" to determine API format
        let is_openai_compatible = self.url.contains("openai");

        if is_openai_compatible {
            // Use OpenAI-compatible format
            self.ask_openai_format(sender, terminate_response_signal).await
        } else {
            // Use native Gemini API format
            self.ask_gemini_format(sender, terminate_response_signal).await
        }
    }
}

impl Gemini {
    async fn ask_openai_format(
        &self,
        sender: UnboundedSender<Event>,
        terminate_response_signal: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse()?);
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.gemini_api_key).parse()?,
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
                            sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
                sender.send(Event::LLMEvent(LLMAnswer::EndAnswer))?;
            }
            Err(e) => {
                eprintln!("Error in OpenAI format request: {:?}", e);
                return Err(Box::new(e));
            }
        }
        Ok(())
    }

    async fn ask_gemini_format(
        &self,
        sender: UnboundedSender<Event>,
        terminate_response_signal: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Format the URL with API key as query parameter
        let url = if self.url.contains("?") {
            format!("{}&key={}", self.url, self.gemini_api_key)
        } else {
            format!("{}?key={}", self.url, self.gemini_api_key)
        };

        // Convert messages to Gemini format
        let contents = self.convert_messages_to_gemini_format();

        // Create request body
        let body: Value = json!({
        "contents": contents,
        "generationConfig": {
        "temperature": 0.7,
        "topK": 32,
        "topP": 0.95,
        "maxOutputTokens": 8192
        }
        });

        //println!("Sending request to URL: {}", url);
        // Make the request
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
        .await?;

        // Process the response
        match response.error_for_status() {
            Ok(res) => {
                sender.send(Event::LLMEvent(LLMAnswer::StartAnswer))?;

                let response_text = res.text().await?;
                let json_response: Value = serde_json::from_str(&response_text)?;

                // Extract text from Gemini response
                if let Some(text) = Self::extract_text_from_gemini_response(&json_response) {
                    // Simulate streaming by breaking text into chunks
                    for (_i, chunk) in text.chars().collect::<Vec<_>>().chunks(5).enumerate() {
                        if terminate_response_signal.load(Ordering::Relaxed) {
                            break;
                        }

                        // Simulate SSE format for compatibility with OpenAI handler
                        let delta_json = json!({
                        "id": format!("chatcmpl-{}", uuid::Uuid::new_v4()),
                        "object": "chat.completion.chunk",
                        "created": chrono::Utc::now().timestamp(),
                        "model": self.model.clone(),
                        "choices": [{
                            "index": 0,
                            "delta": {
                            "content": chunk.iter().collect::<String>()
                        },
                        "finish_reason": null
                    }]
                    });

                        // Convert to SSE format
                        let sse_message = format!("data: {}\n\n", delta_json.to_string());

                        // Process using the same regex as ChatGPT handler
                        let re = Regex::new(r"data:\s(.*)")?;
                        for captures in re.captures_iter(&sse_message) {
                            if let Some(data_json) = captures.get(1) {
                                let answer: Value = serde_json::from_str(data_json.as_str())?;
                                let msg = answer["choices"][0]["delta"]["content"]
                                    .as_str()
                                    .unwrap_or("\n");

                                if msg != "null" {
                                    sender.send(Event::LLMEvent(LLMAnswer::Answer(msg.to_string())))?;
                                }

                                sleep(Duration::from_millis(10)).await;
                            }
                        }
                    }

                    // Send [DONE] message in SSE format
                    let sse_done = "data: [DONE]\n\n";
                    let re = Regex::new(r"data:\s(.*)")?;
                    for captures in re.captures_iter(sse_done) {
                        if let Some(data_json) = captures.get(1) {
                            if data_json.as_str() == "[DONE]" {
                                sender.send(Event::LLMEvent(LLMAnswer::EndAnswer))?;
                            }
                        }
                    }
                } else {
                    // Handle parsing error
                    sender.send(Event::LLMEvent(LLMAnswer::Answer(
                        "Error: Unable to parse Gemini response".to_string()
                    )))?;
                    sender.send(Event::LLMEvent(LLMAnswer::EndAnswer))?;
                }
            }
            Err(e) => {
                let error_message = format!("Error with Gemini request: {:?}", e);
                sender.send(Event::LLMEvent(LLMAnswer::Answer(error_message)))?;
                sender.send(Event::LLMEvent(LLMAnswer::EndAnswer))?;
                return Err(Box::new(e));
            }
        }
        Ok(())
    }

    // Helper function to convert messages to Gemini format
    fn convert_messages_to_gemini_format(&self) -> Vec<Value> {
        let mut contents: Vec<Value> = Vec::new();
        let mut current_role = "";
        let mut current_parts: Vec<Value> = Vec::new();

        // Add system message if needed
        let has_system_message = self.messages.iter().any(|msg| msg["role"] == "system");
        if !has_system_message {
            contents.push(json!({
            "role": "user",
            "parts": [{"text": "You are a helpful assistant."}]
            }));
            contents.push(json!({
            "role": "model",
            "parts": [{"text": "I'm a helpful assistant."}]
            }));
        }

        // Process all messages
        for msg in &self.messages {
            let role = &msg["role"];
            let content = &msg["content"];

            let gemini_role = match role.as_str() {
                "system" => "user", // Gemini doesn't have system role
                "user" => "user",
                "assistant" => "model",
                _ => "user",
            };

            // If role changes, add previous content
            if !current_role.is_empty() && current_role != gemini_role {
                if !current_parts.is_empty() {
                    contents.push(json!({
                    "role": current_role,
                    "parts": current_parts
                    }));
                    current_parts = Vec::new();
                }
            }

            current_role = gemini_role;
            current_parts.push(json!({"text": content}));
        }

        // Add the last role's content
        if !current_role.is_empty() && !current_parts.is_empty() {
            contents.push(json!({
            "role": current_role,
            "parts": current_parts
            }));
        }

        contents
    }

}
