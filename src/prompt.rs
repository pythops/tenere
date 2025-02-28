use arboard::Clipboard;
use ratatui::{
    layout::{Rect},
    style::{Color, Style},
    text::{Text,Span, Line},
    widgets::{Block, BorderType, Borders},
    Frame,
};
use tui_textarea::{CursorMove, TextArea};
use unicode_width::UnicodeWidthStr;

use crate::app::FocusedBlock;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::config::Config;
use crate::llm::LLMBackend;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
}

pub enum LLMStatus {
    Idle,
    Responding,
    Finished,
}

pub struct Prompt<'a> {
    pub mode: Mode,
    pub formatted_prompt: Text<'a>,
    pub editor: TextArea<'a>,
    pub config: Arc<Config>,
    pub llm_status: LLMStatus,
}

impl Default for Prompt<'_> {
    fn default() -> Self {
        let mut editor = TextArea::default();
        editor.remove_line_number();
        editor.set_cursor_line_style(Style::default());
        editor.set_selection_style(Style::default().bg(Color::DarkGray));
        let config = Arc::new(Config::load(None));
        Self {
            mode: Mode::Normal,
            formatted_prompt: Text::raw(""),
            editor,
            config,
            llm_status: LLMStatus::Idle,
        }
    }
}

impl Prompt<'_> {
    
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.formatted_prompt = Text::raw("");
        self.editor.select_all();
        self.editor.cut();
    }

    pub fn height(&self, frame_size: &Rect) -> u16 {
        let prompt_block_max_height = (0.4 * frame_size.height as f32) as u16;

        let height: u16 = 1 + self
            .editor
            .lines()
            .iter()
            .map(|line| 1 + line.width() as u16 / frame_size.width)
            .sum::<u16>();

        std::cmp::min(height, prompt_block_max_height)
    }

    pub fn get_current_model(&self) -> Option<String> {

        match &self.config.llm {
            LLMBackend::ChatGPT => Some(self.config.chatgpt.model.clone()),
            LLMBackend::LLamacpp => self.config.llamacpp.as_ref().map(|c| c.url.clone()),
            LLMBackend::Ollama => self.config.ollama.as_ref().map(|o| o.model.clone()),
        }
    }

    pub fn load_input_file(&self) -> Option<String> {
        if let Some(file_path) = &self.config.input.input_file {
            let path = Path::new(file_path);
            if !path.exists() {
                return None;
            }
            match std::fs::read_to_string(path) {
                Ok(contents) => {
                    Some(contents)
                }
                Err(e) => {
                    eprintln!("Failed to read input file '{}': {}", file_path, e);
                    None
                }
            }
        } else {
            // TODO: send to notifications
            //eprintln!("    No input file specified.");
            None
    }
}

    pub fn handler(
        &mut self,
        key_event: KeyEvent,
        previous_key: KeyCode,
        clipboard: Option<&mut Clipboard>,
    ) {
        match self.mode {
            Mode::Insert => match key_event.code {
                KeyCode::Enter => {
                    self.editor.insert_newline();
                }

                KeyCode::Char(c) => {
                    self.editor.insert_char(c);
                }

                KeyCode::Backspace => {
                    self.editor.delete_char();
                }

                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                }

                // Add support for cursor movement in Insert mode
                KeyCode::Left => {
                    self.editor.move_cursor(CursorMove::Back);
                }

                KeyCode::Right => {
                    self.editor.move_cursor(CursorMove::Forward);
                }

                KeyCode::Up => {
                    self.editor.move_cursor(CursorMove::Up);
                }

                KeyCode::Down => {
                    self.editor.move_cursor(CursorMove::Down);
                }

                KeyCode::Home => {
                    self.editor.move_cursor(CursorMove::Head);
                }

                KeyCode::End => {
                    self.editor.move_cursor(CursorMove::End);
                }

                KeyCode::Delete => {
                    self.editor.delete_next_char();
                }

                _ => {}
            },
            Mode::Command => match key_event.code {
                KeyCode::Enter => {
                    // Execute the command and return to Normal mode
                    // Command execution will be handled in handle_key_events
                    self.mode = Mode::Normal;
                }
                KeyCode::Esc => {
                    // Cancel command entry
                    self.editor.select_all();
                    self.editor.cut();
                    self.mode = Mode::Normal;
                }
                KeyCode::Char(c) => {
                    self.editor.insert_char(c);
                }
                KeyCode::Backspace => {
                    self.editor.delete_char();
                }
                KeyCode::Left => {
                    self.editor.move_cursor(CursorMove::Back);
                }
                KeyCode::Right => {
                    self.editor.move_cursor(CursorMove::Forward);
                }
                // Add more editing keys as needed
                _ => {}
            },
            Mode::Normal | Mode::Visual => match key_event.code {
                KeyCode::Char(':') if self.mode == Mode::Normal => {
                    self.mode = Mode::Command;
                    self.editor.insert_str(":");
                }
                KeyCode::Char('i') => {
                    self.mode = Mode::Insert;
                }

                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                    self.editor.cancel_selection();
                }

                KeyCode::Char('v') => {
                    self.mode = Mode::Visual;
                    self.editor.start_selection();
                }
                 KeyCode::Enter => {
                    if self.editor.lines().join("\n").trim().starts_with(':') {
                        // Return a special value indicating a command should be executed
                        return;
                    }
                }
                KeyCode::Char('h') | KeyCode::Left if key_event.modifiers == KeyModifiers::NONE => {
                    self.editor.move_cursor(CursorMove::Back);
                }

                KeyCode::Char('j') | KeyCode::Down if key_event.modifiers == KeyModifiers::NONE => {
                    self.editor.move_cursor(CursorMove::Down);
                }

                KeyCode::Char('k') | KeyCode::Up if key_event.modifiers == KeyModifiers::NONE => {
                    self.editor.move_cursor(CursorMove::Up);
                }

                KeyCode::Char('l') | KeyCode::Right
                    if key_event.modifiers == KeyModifiers::NONE =>
                {
                    self.editor.move_cursor(CursorMove::Forward);
                }

                KeyCode::Char('m') => {
                    if let Some(input_contents) = self.load_input_file() {
                        self.editor.insert_str(&input_contents); // Insert the contents into the editor
                    }
                }
                KeyCode::Char('w') => match previous_key {
                    KeyCode::Char('d') => {
                        self.editor.delete_next_word();
                    }
                    KeyCode::Char('c') => {
                        self.editor.delete_next_word();
                        self.mode = Mode::Insert;
                    }

                    _ => self.editor.move_cursor(CursorMove::WordForward),
                },

                KeyCode::Char('b') => match previous_key {
                    KeyCode::Char('d') => {
                        self.editor.delete_word();
                    }
                    KeyCode::Char('c') => {
                        self.editor.delete_word();
                        self.mode = Mode::Insert;
                    }

                    _ => self.editor.move_cursor(CursorMove::WordBack),
                },

                KeyCode::Char('$') => match previous_key {
                    KeyCode::Char('d') => {
                        self.editor.delete_line_by_end();
                    }
                    KeyCode::Char('c') => {
                        self.editor.delete_line_by_end();
                        self.mode = Mode::Insert;
                    }
                    _ => self.editor.move_cursor(CursorMove::End),
                },

                KeyCode::Char('0') => match previous_key {
                    KeyCode::Char('d') => {
                        self.editor.delete_line_by_head();
                    }
                    KeyCode::Char('c') => {
                        self.editor.delete_line_by_head();
                        self.mode = Mode::Insert;
                    }
                    _ => self.editor.move_cursor(CursorMove::Head),
                },

                KeyCode::Char('G') => self.editor.move_cursor(CursorMove::Bottom),

                KeyCode::Char('g') => {
                    if previous_key == KeyCode::Char('g') {
                        self.editor.move_cursor(CursorMove::Jump(0, 0))
                    }
                }

                KeyCode::Char('D') => {
                    self.editor.move_cursor(CursorMove::Head);
                    self.editor.delete_line_by_end();
                    self.editor.delete_line_by_head();
                }

                KeyCode::Char('d') => {
                    if previous_key == KeyCode::Char('d') {
                        self.editor.move_cursor(CursorMove::Head);
                        self.editor.delete_line_by_end();
                    }
                }

                KeyCode::Char('c') => {
                    if previous_key == KeyCode::Char('c') {
                        self.editor.move_cursor(CursorMove::Head);
                        self.editor.delete_line_by_end();
                        self.mode = Mode::Insert;
                    }
                }

                KeyCode::Char('C') => {
                    self.editor.delete_line_by_end();
                    self.mode = Mode::Insert;
                }

                KeyCode::Char('x') => {
                    self.editor.delete_next_char();
                }

                KeyCode::Char('a') => {
                    self.editor.move_cursor(CursorMove::Forward);
                    self.mode = Mode::Insert;
                }

                KeyCode::Char('A') => {
                    self.editor.move_cursor(CursorMove::End);
                    self.mode = Mode::Insert;
                }

                KeyCode::Char('o') => {
                    self.editor.move_cursor(CursorMove::End);
                    self.editor.insert_newline();
                    self.mode = Mode::Insert;
                }

                KeyCode::Char('O') => {
                    self.editor.move_cursor(CursorMove::Head);
                    self.editor.insert_newline();
                    self.editor.move_cursor(CursorMove::Up);
                    self.mode = Mode::Insert;
                }

                KeyCode::Char('I') => {
                    self.editor.move_cursor(CursorMove::Head);
                    self.mode = Mode::Insert;
                }

                KeyCode::Char('y') => {
                    self.editor.copy();
                    if let Some(clipboard) = clipboard {
                        let text = self.editor.yank_text();
                        let _ = clipboard.set_text(text);
                    }
                }

                KeyCode::Char('p') => {
                    if !self.editor.paste() {
                        if let Some(clipboard) = clipboard {
                            if let Ok(text) = clipboard.get_text() {
                                self.editor.insert_str(text);
                            }
                        }
                    }
                }

                KeyCode::Char('u') => {
                    self.editor.undo();
                }

                KeyCode::Home => {
                    self.editor.move_cursor(CursorMove::Head);
                }

                KeyCode::End => {
                    self.editor.move_cursor(CursorMove::End);
                }



                _ => {}
            },
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused_block: &FocusedBlock) {
        // Calculate the height of the prompt box (information bar)
        let prompt_height = self.height(&area);
        let _prompt_area = Rect {
            x: area.x,
            y: area.height.saturating_sub(prompt_height), // Position at the bottom
            width: area.width,
            height: prompt_height,
        };

        // Retrieve the current model from the configuration
        let model_info = self.get_current_model().unwrap_or_else(|| "Unknown".to_string());

        // Determine the mode-specific text and colors
        let (mode_text, mode_fg, mode_bg) = match self.mode {
            Mode::Insert => ("INSERT", Color::White, Color::Green),
            Mode::Normal => ("NORMAL", Color::White, Color::Blue),
            Mode::Visual => ("VISUAL", Color::White, Color::Yellow),
            Mode::Command => ("COMMAND", Color::White, Color::Green),
        };

        // Get the cursor position (line and column)
        let cursor_pos = format!(
            "Ln {}, Col {}",
            self.editor.cursor().0 + 1, // Line number (1-based)
            self.editor.cursor().1 + 1  // Column number (1-based)
        );

        // Define a color for the model segment
        let model_bg = Color::Cyan;

        // Define a color for the cursor position segment
        let cursor_bg = Color::Rgb(70, 70, 70);  

        // Define the LLM status segment
        let (llm_status_text, llm_status_fg, llm_status_bg) = match self.llm_status {
            LLMStatus::Idle => ("IDLE ", Color::White, Color::DarkGray),
            LLMStatus::Responding => (" 󰟃   ", Color::White, Color::Yellow),
            LLMStatus::Finished => ("IDLE ", Color::White, Color::Green),
        };

        let title = if self.mode == Mode::Command {
            // In Command mode, show the command being typed
            let command_text = self.editor.lines().get(0).unwrap_or(&String::new()).clone();
            Line::from(vec![
                Span::styled(
                    format!("{} ", mode_text),
                    Style::default().fg(mode_fg).bg(mode_bg),
                ),
                Span::styled(
                    "\u{E0B0} ",
                    Style::default().fg(mode_bg).bg(Color::Black),
                ),
                Span::styled(
                    command_text,
                    Style::default().fg(Color::White).bg(Color::Black),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled(
                    format!("{} ", mode_text),
                    Style::default().fg(mode_fg).bg(mode_bg),
                ),
                Span::styled("\u{E0B0} ", Style::default().fg(mode_bg).bg(model_bg)),
                Span::styled(format!("{}", model_info), Style::default().fg(Color::White).bg(model_bg)),
                Span::styled("\u{E0B0} ", Style::default().fg(model_bg).bg(cursor_bg)),
                Span::styled(cursor_pos, Style::default().fg(Color::White).bg(cursor_bg)),
                Span::styled("\u{E0B0} ", Style::default().fg(cursor_bg).bg(llm_status_bg)),
                Span::styled(llm_status_text, Style::default().fg(llm_status_fg).bg(llm_status_bg)),
                Span::styled("\u{E0B0}", Style::default().fg(llm_status_bg).bg(Color::Black)),
            ])
        };

        // Create the Block widget with borders and the styled title
        let prompt_block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style({
                if *focused_block == FocusedBlock::Prompt {
                    match self.mode {
                        Mode::Insert => Style::default().fg(Color::Green),
                        Mode::Normal => Style::default(),
                        Mode::Visual => Style::default().fg(Color::Yellow),
                        Mode::Command => Style::default().fg(Color::Magenta),
                    }
                } else {
                    Style::default()
                }
            })
            .border_type({
                if *focused_block == FocusedBlock::Prompt {
                    BorderType::Thick
                } else {
                    BorderType::Rounded
                }
            });

        // Render the Block widget
        frame.render_widget(prompt_block, area);

        // Render the editor content inside the inner area of the block
        frame.render_widget(
            &self.editor,
            area.inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            }),
        );
    }
}
