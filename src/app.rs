use std;

use crossterm::event::KeyCode;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::gpt;

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
    pub gpt: gpt::GPT,
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
            gpt: gpt::GPT::new(),
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
        //TODO: Make scroll stops when reaches the top or the bottom
        let prompt = {
            if prompt_height > max_prompt_height {
                let mut scroll: i32 =
                    prompt_height as i32 - max_prompt_height as i32 + 2 + self.scroll;
                if scroll < 0 {
                    scroll = 0;
                    self.scroll = 0;
                }

                println!("{}, {}", scroll, self.scroll);
                if let FocusedBlock::Chat = self.focused_block {
                    scroll = 0;
                }

                Paragraph::new(self.input.as_ref())
                    .wrap(Wrap { trim: false })
                    .scroll((scroll as u16, 0))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .style(Style::default())
                            .border_type(BorderType::Rounded)
                            .border_style(match self.focused_block {
                                FocusedBlock::Prompt => Style::default().fg(Color::Green),
                                _ => Style::default(),
                            }),
                    )
            } else {
                Paragraph::new(self.input.as_ref())
                    .wrap(Wrap { trim: false })
                    .style(Style::default())
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .style(Style::default())
                            .border_type(BorderType::Rounded)
                            .border_style(match self.focused_block {
                                FocusedBlock::Prompt => Style::default().fg(Color::Green),
                                _ => Style::default(),
                            }),
                    )
            }
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
                prompt_block.y + prompt_height - 3,
            ),
        }

        // Messages block
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

            if messages_height > chat_height {
                let mut scroll = messages_height as i32 - chat_height as i32 + self.scroll;
                if scroll < 0 {
                    scroll = 0;
                    self.scroll = 0;
                }
                if let FocusedBlock::Prompt = self.focused_block {
                    scroll = 0;
                }

                Paragraph::new(messages)
                    .scroll((scroll as u16, 0))
                    .wrap(Wrap { trim: false })
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .style(Style::default())
                            .border_type(BorderType::Rounded)
                            .border_style(match self.focused_block {
                                FocusedBlock::Chat => Style::default().fg(Color::Green),
                                _ => Style::default(),
                            }),
                    )
            } else {
                Paragraph::new(messages).wrap(Wrap { trim: false }).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default())
                        .border_type(BorderType::Rounded)
                        .border_style(match self.focused_block {
                            FocusedBlock::Chat => Style::default().fg(Color::Green),
                            _ => Style::default(),
                        }),
                )
            }
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
`Tab`    : Switch between the prompt and the chat history
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
