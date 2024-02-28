use crate::chatgpt::ChatGPT;
use crate::config::Config;
use crate::event::Event;
use crate::llamacpp::LLamacpp;
use crate::ollama::Ollama;
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::atomic::AtomicBool;
use strum_macros::Display;
use strum_macros::EnumIter;
use tokio::sync::mpsc::UnboundedSender;

use std::sync::Arc;

#[async_trait]
pub trait LLM: Send + Sync {
    async fn ask(
        &self,
        sender: UnboundedSender<Event>,
        terminate_response_signal: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    fn append_chat_msg(&mut self, msg: String, role: LLMRole);
    fn clear(&mut self);
}

#[derive(Clone, Debug)]
pub enum LLMAnswer {
    StartAnswer,
    Answer(String),
    EndAnswer,
}

#[derive(EnumIter, Display, Debug)]
#[strum(serialize_all = "lowercase")]
pub enum LLMRole {
    ASSISTANT,
    SYSTEM,
    USER,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LLMBackend {
    ChatGPT,
    LLamacpp,
    Ollama,
}

pub struct LLMModel;

impl LLMModel {
    pub async fn init(model: &LLMBackend, config: Arc<Config>) -> Box<dyn LLM> {
        match model {
            LLMBackend::ChatGPT => Box::new(ChatGPT::new(config.chatgpt.clone().unwrap())),
            LLMBackend::LLamacpp => Box::new(LLamacpp::new(config.llamacpp.clone().unwrap())),
            LLMBackend::Ollama => Box::new(Ollama::new(config.ollama.clone().unwrap())),
        }
    }
}
