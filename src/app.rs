use crate::prompt::Prompt;
use std;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;

use crate::notification::Notification;
use crate::spinner::Spinner;
use crate::{config::Config, formatter::Formatter};
use arboard::Clipboard;
use ratatui::text::{Line, Text};

use std::sync::Arc;

pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, PartialEq)]
pub enum FocusedBlock {
    Prompt,
    Chat,
    History,
    Preview,
    Help,
}

#[derive(Debug, Default, Clone)]
pub struct History<'a> {
    pub show: bool,
    pub index: usize,
    pub chat: Vec<Vec<String>>,
    pub formatted_chat: Vec<Text<'a>>,
    pub scroll: u16,
    pub length: u16,
}

#[derive(Debug, Default)]
pub struct Chat<'a> {
    pub messages: Vec<String>,
    pub formatted_chat: Text<'a>,
    pub scroll: u16,
    pub length: u16,
}

#[derive(Debug, Default)]
pub struct Answer<'a> {
    pub answer: String,
    pub formatted_answer: Text<'a>,
}

pub struct App<'a> {
    pub running: bool,
    pub prompt: Prompt<'a>,
    pub chat: Chat<'a>,
    pub focused_block: FocusedBlock,
    pub llm_messages: Vec<HashMap<String, String>>,
    pub answer: Answer<'a>,
    pub history: History<'a>,
    pub notifications: Vec<Notification>,
    pub spinner: Spinner,
    pub terminate_response_signal: Arc<AtomicBool>,
    pub clipboard: Option<Clipboard>,
    pub config: Arc<Config>,
    pub formatter: &'a Formatter<'a>,
}

impl<'a> App<'a> {
    pub fn new(config: Arc<Config>, formatter: &'a Formatter<'a>) -> Self {
        Self {
            running: true,
            prompt: Prompt::default(),
            chat: Chat::default(),
            focused_block: FocusedBlock::Prompt,
            llm_messages: Vec::new(),
            answer: Answer::default(),
            history: History::default(),
            notifications: Vec::new(),
            spinner: Spinner::default(),
            terminate_response_signal: Arc::new(AtomicBool::new(false)),
            clipboard: Clipboard::new().ok(),
            config,
            formatter,
        }
    }

    pub fn tick(&mut self) {
        self.notifications.retain(|n| n.ttl > 0);
        self.notifications.iter_mut().for_each(|n| n.ttl -= 1);

        if self.spinner.active {
            self.chat.formatted_chat.lines.pop();
            self.chat
                .formatted_chat
                .lines
                .push(Line::raw(format!("🤖: Waiting {}", self.spinner.draw())));
            self.spinner.update();
        }
    }
}
