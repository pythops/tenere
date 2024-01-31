use crate::chatgpt::ChatGPT;
use crate::config::Config;
use crate::event::Event;
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
