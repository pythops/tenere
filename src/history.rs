use core::str;
use std::{fs, path::PathBuf};

use tokio::sync::mpsc::UnboundedSender;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::{
    app::FocusedBlock,
    event::Event,
    notification::{Notification, NotificationLevel},
};

#[derive(Debug, Default, Clone)]
pub struct Preview<'a> {
    pub text: Vec<Text<'a>>,
    pub scroll: usize,
}

#[derive(Debug, Default, Clone)]
pub struct History<'a> {
    state: ListState,
    pub text: Vec<Vec<String>>,
    pub preview: Preview<'a>,
}

impl History<'_> {
    pub fn new() -> Self {
        Self {
            state: ListState::default(),
            text: Vec::new(),
            preview: Preview::default(),
        }
    }

    pub fn move_to_bottom(&mut self) {
        if !self.text.is_empty() {
            self.state.select(Some(self.text.len() - 1));
        }
    }

    pub fn move_to_top(&mut self) {
        if !self.text.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn scroll_down(&mut self) {
        if self.text.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i < self.text.len() - 1 {
                    i + 1
                } else {
                    i
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    pub fn scroll_up(&mut self) {
        if self.text.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.state.select(Some(i));
    }

    // check if data directory for the application exists, else it will create it
    pub fn check_data_directory_exists(&self, sender: UnboundedSender<Event>) {
        if let Some(data_directory) = dirs::data_dir() {
            let target_directory = data_directory.join("tenere");

            if !target_directory.exists() {
                if let Err(e) = fs::create_dir_all(target_directory) {
                    let notif = Notification::new(e.to_string(), NotificationLevel::Error);
                    sender.send(Event::Notification(notif)).unwrap();
                }
            }
        }
    }

    // load chat in the history from data directory
    pub fn load_history(&mut self, sender: UnboundedSender<Event>) {
        let directory_path: PathBuf = dirs::data_dir().unwrap().join("tenere");

        if let Ok(paths) = fs::read_dir(directory_path.clone()) {
            // foreach archive file we add it to history
            for path in paths {
                if path.as_ref().unwrap().file_type().unwrap().is_file() {
                    self.load_chat_from_file(path.unwrap().path().to_str().unwrap());
                }
            }

            let notif = Notification::new("History loaded".to_string(), NotificationLevel::Info);

            sender.send(Event::Notification(notif)).unwrap();
        }
    }

    /// Add to history the archive file if exists
    pub fn load_chat_from_file(&mut self, archive_file_name: &str) {
        if let Ok(text) = std::fs::read_to_string(archive_file_name) {
            // push full conversation in preview
            self.preview.text.push(Text::from(text.clone()));
            // get first line of the conversation
            let first_line: String = text.lines().next().unwrap_or("").to_string();
            self.text.push(vec![first_line]);
        }
    }

    // call after adding new chat in history (Starting a new chat)
    // with the index of the chat in history to save
    pub fn save(&mut self, chat_index_in_history: usize, sender: UnboundedSender<Event>) {
        let file_name = format!("tenere.archive-{}", chat_index_in_history);
        let file_path: PathBuf = dirs::data_dir().unwrap().join("tenere").join(file_name);

        if !self.text.is_empty() {
            match std::fs::write(file_path.clone(), self.text[chat_index_in_history].join("")) {
                Ok(_) => {
                    let notif =
                        Notification::new("Chat saved".to_string(), NotificationLevel::Info);

                    sender.send(Event::Notification(notif)).unwrap();
                }
                Err(e) => {
                    let notif = Notification::new(e.to_string(), NotificationLevel::Error);

                    sender.send(Event::Notification(notif)).unwrap();
                }
            }
        }
    }

    pub fn render(&mut self, frame: &mut Frame, focused_block: &FocusedBlock) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Percentage(90),
                Constraint::Fill(1),
            ])
            .split(frame.area())[1];

        let block = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Percentage(90),
                Constraint::Fill(1),
            ])
            .split(layout)[1];

        if !self.text.is_empty() && self.state.selected().is_none() {
            *self.state.offset_mut() = 0;
            self.state.select(Some(0));
        }

        let (history_block, preview_block) = {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(block);
            (chunks[0], chunks[1])
        };

        let items = self
            .text
            .iter()
            .map(|chat| match chat.first() {
                Some(v) => ListItem::new(v.to_owned()),
                None => ListItem::new(""),
            })
            .collect::<Vec<ListItem>>();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" History ")
                    .title_style(match focused_block {
                        FocusedBlock::History => Style::default().bold(),
                        _ => Style::default(),
                    })
                    .title_alignment(Alignment::Center)
                    .style(Style::default())
                    .border_style(match focused_block {
                        FocusedBlock::History => Style::default().fg(Color::Green),
                        _ => Style::default(),
                    }),
            )
            .highlight_style(Style::default().bg(Color::DarkGray));

        let preview = Paragraph::new(match self.state.selected() {
            Some(i) => self.preview.text[i].clone(),
            None => Text::raw(""),
        })
        .wrap(Wrap { trim: false })
        .scroll((self.preview.scroll as u16, 0))
        .block(
            Block::default()
                .title(" Preview ")
                .title_style(match focused_block {
                    FocusedBlock::Preview => Style::default().bold(),
                    _ => Style::default(),
                })
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .style(Style::default())
                .border_style(match focused_block {
                    FocusedBlock::Preview => Style::default().fg(Color::Green),
                    _ => Style::default(),
                }),
        );

        frame.render_widget(Clear, block);
        frame.render_widget(preview, preview_block);
        frame.render_stateful_widget(list, history_block, &mut self.state);
    }
}
