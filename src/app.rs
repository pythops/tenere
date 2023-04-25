use std;
use std::collections::HashMap;

use crate::config::Config;
use crate::notification::Notification;
use crossterm::event::KeyCode;

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
}

#[derive(Debug)]
pub struct App {
    pub prompt: String,
    pub mode: Mode,
    pub running: bool,
    pub chat: Vec<String>,
    pub scroll: i32,
    pub previous_key: KeyCode,
    pub focused_block: FocusedBlock,
    pub show_help_popup: bool,
    pub llm_messages: Vec<HashMap<String, String>>,
    pub history: Vec<Vec<String>>,
    pub show_history_popup: bool,
    pub history_thread_index: usize,
    pub config: Arc<Config>,
    pub notifications: Vec<Notification>,
}

impl App {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            running: true,
            prompt: String::from(">_ "),
            mode: Mode::Normal,
            chat: Vec::new(),
            scroll: 0,
            previous_key: KeyCode::Null,
            focused_block: FocusedBlock::Prompt,
            show_help_popup: false,
            llm_messages: Vec::new(),
            history: Vec::new(),
            show_history_popup: false,
            history_thread_index: 0,
            config,
            notifications: Vec::new(),
        }
    }

    pub fn tick(&mut self) {
        self.notifications.retain(|n| n.ttl > 0);
        self.notifications.iter_mut().for_each(|n| n.ttl -= 1);
    }
}
