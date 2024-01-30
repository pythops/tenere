use crate::chatgpt::ChatGPT;
use crate::config::Config;
use crate::event::Event;
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc::UnboundedSender;

use std::sync::Arc;

#[async_trait]
pub trait LLM: Send + Sync {
    async fn ask(
        &self,
        sender: UnboundedSender<Event>,
        terminate_response_signal: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    fn append_chat_msg(&mut self, chat: String);
    fn clear(&mut self);
}

#[derive(Clone, Debug)]
pub enum LLMAnswer {
    StartAnswer,
    Answer(String),
    EndAnswer,
}

#[derive(Deserialize, Debug)]
pub enum LLMBackend {
    ChatGPT,
}

pub struct LLMModel;

impl LLMModel {
    pub async fn init(model: &LLMBackend, config: Arc<Config>) -> impl LLM {
        match model {
            LLMBackend::ChatGPT => ChatGPT::new(config.chatgpt.clone()),
        }
    }
}
