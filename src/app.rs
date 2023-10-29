use std;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;

use crate::config::Config;
use crate::notification::Notification;
use crate::spinner::Spinner;
use crossterm::event::KeyCode;
use tui::text::{Line, Text};

use std::sync::Arc;

pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub enum Mode {
    Normal,
    Insert,
}

#[derive(Debug)]
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
}

#[derive(Debug, Default)]
pub struct Chat<'a> {
    pub messages: Vec<String>,
    pub formatted_chat: Text<'a>,
}

#[derive(Debug)]
pub struct Prompt {
    pub message: String,
}

impl Default for Prompt {
    fn default() -> Self {
        Self {
            message: String::from(">_ "),
        }
    }
}

#[derive(Debug, Default)]
pub struct Answer<'a> {
    pub answer: String,
    pub formatted_answer: Text<'a>,
}

pub struct App<'a> {
    pub running: bool,
    pub scroll: usize,
    pub prompt: Prompt,
    pub mode: Mode,
    pub chat: Chat<'a>,
    pub previous_key: KeyCode,
    pub focused_block: FocusedBlock,
    pub llm_messages: Vec<HashMap<String, String>>,
    pub answer: Answer<'a>,
    pub history: History<'a>,
    pub config: Arc<Config>,
    pub notifications: Vec<Notification>,
    pub spinner: Spinner,
    pub terminate_response_signal: Arc<AtomicBool>,
}

impl<'a> App<'a> {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            running: true,
            scroll: 0,
            prompt: Prompt::default(),
            mode: Mode::Normal,
            chat: Chat::default(),
            previous_key: KeyCode::Null,
            focused_block: FocusedBlock::Prompt,
            llm_messages: Vec::new(),
            answer: Answer::default(),
            history: History::default(),
            config,
            notifications: Vec::new(),
            spinner: Spinner::default(),
            terminate_response_signal: Arc::new(AtomicBool::new(false)),
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
