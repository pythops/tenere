use ansi_to_tui::IntoText;
use std;
use std::collections::HashMap;

use crossterm::event::KeyCode;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

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
}

#[derive(Debug)]
pub struct App {
    pub input: String,
    pub mode: Mode,
    pub running: bool,
    pub messages: Vec<String>,
    pub scroll: i32,
    pub previous_key: KeyCode,
    pub focused_block: FocusedBlock,
    pub show_help_popup: bool,
    pub history: Vec<HashMap<String, String>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            input: String::from(">_ "),
            mode: Mode::Normal,
            messages: Vec::new(),
            scroll: 0,
            previous_key: KeyCode::Null,
            focused_block: FocusedBlock::Prompt,
            show_help_popup: false,
            history: Vec::new(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&self) {}

    pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_y) / 2),
                    Constraint::Percentage(percent_y),
                    Constraint::Percentage((100 - percent_y) / 2),
                ]
                .as_ref(),
            )
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_x) / 2),
                    Constraint::Percentage(percent_x),
                    Constraint::Percentage((100 - percent_x) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1]
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        // Layout
        let app_area = frame.size();

        // prompt height can grow till 40% of the frame height
        let max_prompt_height = (0.4 * app_area.height as f32) as u16;
        let prompt_height = {
            let mut height: u16 = 3;
            for line in self.input.lines() {
                height += 1;
                height += line.width() as u16 / app_area.width;
            }
            height
        };

        // chat height is the frame height minus the prompt height
        let max_chat_height = app_area.height - max_prompt_height - 3;
        let chat_height = app_area.height - prompt_height - 3;

        let (chat_block, prompt_block, mode_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(std::cmp::max(chat_height, max_chat_height)),
                        Constraint::Length(std::cmp::min(prompt_height, max_prompt_height)),
                        Constraint::Length(2),
                    ]
                    .as_ref(),
                )
                .split(frame.size());
            (chunks[0], chunks[1], chunks[2])
        };

        // prompt block
        //TODO: show scroll bar
        let prompt = {
            let mut scroll = 0;
            let height_diff = prompt_height as i32 - max_prompt_height as i32;
            if let FocusedBlock::Prompt = self.focused_block {
                if height_diff + self.scroll >= 0 {
                    scroll = height_diff + self.scroll;
                }

                // scroll up case
                if height_diff > 0 && -self.scroll > height_diff {
                    self.scroll = -height_diff;
                }

                // Scroll down case
                // 2 empty lines
                if height_diff > 0 && self.scroll >= 2 {
                    self.scroll = 2
                }
            }
            Paragraph::new(self.input.as_ref())
                .wrap(Wrap { trim: false })
                .scroll((scroll as u16, 0))
                .style(Style::default())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default())
                        .border_type(match self.focused_block {
                            FocusedBlock::Prompt => BorderType::Thick,
                            _ => BorderType::Rounded,
                        })
                        .border_style(match self.focused_block {
                            FocusedBlock::Prompt => Style::default().fg(Color::Green),
                            _ => Style::default(),
                        }),
                )
        };

        match self.mode {
            Mode::Normal => {}

            // TODO: set the cursor position
            Mode::Insert => frame.set_cursor(
                prompt_block.x
                    + {
                        let last_line = self.input.lines().last().unwrap_or("");
                        let mut width = last_line.len() as u16;
                        if last_line.len() as u16 > app_area.width {
                            let last_word = last_line.rsplit(' ').last().unwrap_or("");
                            width =
                                last_line.width() as u16 % app_area.width + last_word.len() as u16;
                        }
                        width
                    }
                    + 1,
                prompt_block.y + std::cmp::min(prompt_height, max_prompt_height) - 3,
            ),
        }

        // Chat block
        let chat = {
            let messages: String = self.messages.iter().map(|m| m.to_string()).collect();

            let messages_height = {
                let mut height: u16 = 0;
                for msg in &self.messages {
                    height += 1;
                    for line in msg.lines() {
                        height += 1;
                        height += line.width() as u16 / app_area.width;
                    }
                }
                height
            };

            let mut scroll = 0;
            let height_diff = messages_height as i32 - chat_height as i32;
            if height_diff > 0 {
                scroll = height_diff;
            }
            if let FocusedBlock::Chat = self.focused_block {
                if height_diff + self.scroll >= 0 {
                    scroll = height_diff + self.scroll;
                }

                // // scroll up case
                if height_diff > 0 && -self.scroll > height_diff {
                    self.scroll = -height_diff;
                }
                //
                // // Scroll down case
                if height_diff > 0 && self.scroll > 0 {
                    self.scroll = 0;
                }
            }

            Paragraph::new({
                termimad::inline(messages.as_str())
                    .to_string()
                    .into_text()
                    .unwrap_or(Text::from(messages))
            })
            .scroll((scroll as u16, 0))
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default())
                    .border_type(match self.focused_block {
                        FocusedBlock::Chat => BorderType::Thick,
                        _ => BorderType::Rounded,
                    })
                    .border_style(match self.focused_block {
                        FocusedBlock::Chat => Style::default().fg(Color::Green),
                        _ => Style::default(),
                    }),
            )
        };

        // Mode blokc
        let mode = Paragraph::new({
            match self.mode {
                Mode::Normal => "Mode: Normal",
                Mode::Insert => "Mode: Insert",
            }
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default())
                .border_type(BorderType::Rounded),
        );

        // Draw
        frame.render_widget(chat, chat_block);
        frame.render_widget(prompt, prompt_block);
        frame.render_widget(mode, mode_block);

        if self.show_help_popup {
            let help = "
`i`      : Switch to Insert mode
`Esc`    : Switch to Normal mode
`dd`     : Clear the prompt
`ctrl+l` : Clear the prompt AND the chat
`Tab`    : Switch the focus between the prompt and the chat history
`j`      : Scroll down
`k`      : Scroll up
`q`      : Quit
`h`      : show help
            ";

            let block = Paragraph::new(help).wrap(Wrap { trim: false }).block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default())
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Yellow)),
            );
            let area = Self::centered_rect(75, 25, app_area);
            frame.render_widget(Clear, area); //this clears out the background
            frame.render_widget(block, area);
        }
    }
}
