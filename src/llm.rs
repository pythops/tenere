use crate::config::Config;
use crate::gpt::GPT;
use std::collections::HashMap;

use std::sync::Arc;
pub trait LLM: Send + Sync {
    fn ask(
        &self,
        chat_messages: Vec<HashMap<String, String>>,
    ) -> Result<String, Box<dyn std::error::Error>>;
}

pub enum LLMBackend {
    ChatGPT,
}

pub struct LLMModel {}

impl LLMModel {
    pub fn init(model: LLMBackend, config: Arc<Config>) -> impl LLM {
        match model {
            LLMBackend::ChatGPT => GPT::new(config.gpt.clone()),
        }
    }
}
