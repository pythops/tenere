use crate::chatgpt::ChatGPT;
use crate::config::Config;
use serde::Deserialize;
use std::collections::HashMap;

use std::sync::Arc;
pub trait LLM: Send + Sync {
    fn ask(
        &self,
        chat_messages: Vec<HashMap<String, String>>,
    ) -> Result<String, Box<dyn std::error::Error>>;
}

#[derive(Deserialize, Debug)]
pub enum LLMBackend {
    ChatGPT,
}

pub struct LLMModel {}

impl LLMModel {
    pub fn init(model: LLMBackend, config: Arc<Config>) -> impl LLM {
        match model {
            LLMBackend::ChatGPT => ChatGPT::new(config.chatgpt.clone()),
        }
    }
}
