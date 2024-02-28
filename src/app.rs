use crate::history::History;
use crate::prompt::Prompt;
use crate::{chat::Chat, help::Help};
use std;
use std::sync::atomic::AtomicBool;

use crate::notification::Notification;
use crate::spinner::Spinner;
use crate::{config::Config, formatter::Formatter};
use arboard::Clipboard;
use crossterm::event::KeyCode;
use ratatui::text::Line;

use std::sync::Arc;

pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, PartialEq)]
pub enum FocusedBlock {
    Prompt,
    Chat,
    History,
    Preview,
    Help,
}

pub struct App<'a> {
    pub running: bool,
    pub prompt: Prompt<'a>,
    pub chat: Chat<'a>,
    pub focused_block: FocusedBlock,
    pub history: History<'a>,
    pub notifications: Vec<Notification>,
    pub spinner: Spinner,
    pub terminate_response_signal: Arc<AtomicBool>,
    pub clipboard: Option<Clipboard>,
    pub help: Help,
    pub previous_key: KeyCode,
    pub config: Arc<Config>,
    pub formatter: &'a Formatter<'a>,
}

impl<'a> App<'a> {
    pub fn new(config: Arc<Config>, formatter: &'a Formatter<'a>) -> Self {
        Self {
            running: true,
            prompt: Prompt::default(),
            chat: Chat::new(),
            focused_block: FocusedBlock::Prompt,
            history: History::new(),
            notifications: Vec::new(),
            spinner: Spinner::default(),
            terminate_response_signal: Arc::new(AtomicBool::new(false)),
            clipboard: Clipboard::new().ok(),
            help: Help::new(),
            previous_key: KeyCode::Null,
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
