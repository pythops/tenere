use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Serialize, Deserialize, Debug)]
struct LLMResp {
    status: u16,
    body: String,
}

trait LLMConfig {
    fn base_url(&self) -> &str;
    fn api_key(&self) -> &str;
    fn model_type(&self) -> &str;
}

#[async_trait]
trait LLMConversation {
    async fn new(client: Client, config: impl LLMConfig) -> Self;
    async fn ask(&self, prompt: &str) -> LLMResp;
    async fn upload_file(&self, file_path: PathBuf) -> LLMResp;

    fn get_config(&self) -> impl LLMConfig;
    fn get_client(&self) -> Client;
}

#[derive(Debug)]
struct ClaudeConversation {
    id: String,
    client: Client,
    config: ClaudeConfig,
}

#[derive(Debug)]
struct ClaudeConfig {
    base_url: String,
    api_key: String,
    model_type: String,
}

impl LLMConfig for ClaudeConfig {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn api_key(&self) -> &str {
        &self.api_key
    }

    fn model_type(&self) -> &str {
        &self.model_type
    }
}

impl LLMConversation for ClaudeConversation {
    fn get_config(&self) {
        self.config
    }

    fn get_client(&self) {
        self.client
    }

    async fn new(client: Client, config: impl LLMConfig) -> Self {
        let client = client.clone().bearer_auth(config.api_key());
        let resp = client
            .post(&format!("{}/conversations", config.base_url()))
            .send()
            .await
            .unwrap()
            .json::<HashMap<String, String>>()
            .await
            .unwrap();

        Self {
            id: resp["id"].clone(),
            config,
            client,
        }
    }

    async fn ask(&self, prompt: &str) -> LLMResp {
        let body = serde_json::json!({
            "content": prompt,
        });

        let resp = self
            .client
            .post(&format!(
                "{}/conversations/{}/message",
                self.client.base_url().unwrap(),
                self.id
            ))
            .bearer_auth(self.client.auth_value().unwrap())
            .json(&body)
            .send()
            .await
            .unwrap()
            .json::<LLMResp>()
            .await
            .unwrap();

        resp
    }

    async fn upload_file(&self, conv: &Conversation, file_path: PathBuf) -> LLMResp {
        let mut file = File::open(file_path).await.unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await.unwrap();

        let resp = conv
            .client
            .post(&format!(
                "{}/conversations/{}/file",
                self.config.base_url(),
                conv.id
            ))
            .body(buffer)
            .send()
            .await
            .unwrap();

        LLMResp {
            status: resp.status().as_u16(),
            body: String::new(),
        }
    }

}

#[tokio::main]
async fn main() {
    let config = ClaudeConfig {
        base_url: "https://api.claude.ai/v1".to_string(),
        api_key: "your_api_key_here".to_string(),
        model_type: "claude-v1".to_string(),
    };
    let client = Client::new();

    let conv = claude.create_conv().await;
    let resp = conv.ask("Hello, Claude!").await;
    println!("{}", resp.body);

    let file_resp = claude
        .upload_file(&conv, PathBuf::from("example_file.txt"))
        .await;
    println!("File upload status: {}", file_resp.status);
}
