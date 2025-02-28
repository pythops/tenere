use std::{rc::Rc, sync::atomic::AtomicBool, fs::OpenOptions, io::Write};
use std::sync::Arc;

use ratatui::{
    layout::Rect,
    text::Text,
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

use crate::{formatter::Formatter, llm::LLMAnswer};
use crate::prompt::Prompt;
use crate::config::Config;

// Define CommandResult at module level with pub visibility
#[derive(Debug)]
pub enum CommandResult {
    Success(String),
    Error(String),
    Exit,
}

#[derive(Debug, Clone, Default)]
pub struct Answer<'a> {
    pub plain_answer: String,
    pub formatted_answer: Text<'a>,
}

#[derive(Debug, Clone)]
pub struct Chat<'a> {
    pub plain_chat: Vec<String>,
    pub formatted_chat: Text<'a>,
    pub answer: Answer<'a>,
    pub scroll: u16,
    area_height: u16,
    area_width: u16,
    pub automatic_scroll: Rc<AtomicBool>,
    pub config: Arc<Config>,
}

impl Default for Chat<'_> {
    fn default() -> Self {
        Self {
            plain_chat: Vec::new(),
            formatted_chat: Text::raw(""),
            answer: Answer::default(),
            scroll: 0,
            area_height: 0,
            area_width: 0,
            automatic_scroll: Rc::new(AtomicBool::new(true)),
            config: Arc::new(Config::load(None)),
        }
    }
}

impl Chat<'_> {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            plain_chat: Vec::new(),
            formatted_chat: Text::raw(""),
            answer: Answer::default(),
            scroll: 0,
            area_height: 0,
            area_width: 0,
            automatic_scroll: Rc::new(AtomicBool::new(true)),
            config,
        }
    }

    pub fn handle_answer(&mut self, event: LLMAnswer, formatter: &Formatter) {
        // First match: Borrow answer instead of moving it
        match event {
            LLMAnswer::StartAnswer => {
                self.formatted_chat.lines.pop();
            }
            LLMAnswer::Answer(ref answer) => {  // Use ref to borrow answer
                self.answer.plain_answer.push_str(answer.as_str());
                self.answer.formatted_answer = formatter.format(format!("🤖: {}", &self.answer.plain_answer).as_str());
            }
            LLMAnswer::EndAnswer => {
                self.formatted_chat.extend(self.answer.formatted_answer.clone());
                self.formatted_chat.extend(Text::raw("\n"));
                let full_answer = format!("🤖: {}", self.answer.plain_answer);
                self.plain_chat.push(full_answer);
                self.answer = Answer::default();
            }
        }

        // Second match: Now safe to move answer
        if let Some(output_path) = self.config.input.output_file.as_deref() {
            match event {
                LLMAnswer::StartAnswer => {
                    let _ = std::fs::write(output_path, "").expect("Failed to clear output file");
                }
                LLMAnswer::Answer(answer) => {  // Move answer here
                    let mut file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(output_path)
                        .expect("Failed to open output file");
                    file.write_all(answer.as_bytes())
                        .expect("Failed to write to output file");
                    file.write_all(b"\n")
                        .expect("Failed to write newline to output file");
                }
                LLMAnswer::EndAnswer => {
                    let full_answer = format!("🤖: {}", self.answer.plain_answer);
                    let mut file = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(output_path)
                        .expect("Failed to open output file");
                    file.write_all(full_answer.as_bytes())
                        .expect("Failed to write to output file");
                    file.write_all(b"\n")
                        .expect("Failed to write newline to output file");
                }
            }
        }
    }

    pub fn height(&self) -> usize {
        let mut chat = self.formatted_chat.clone();

        chat.extend(self.answer.formatted_answer.clone());

        let nb_lines = chat.lines.len() + 3;
        chat.lines.iter().fold(nb_lines, |acc, line| {
            acc + line.width() / self.area_width as usize
        })
    }

    pub fn move_to_bottom(&mut self) {
        self.scroll = (self.formatted_chat.height() + self.answer.formatted_answer.height())
            .saturating_sub((self.area_height - 2).into()) as u16;
    }

    pub fn move_to_top(&mut self) {
        self.scroll = 0;
    }

    pub fn execute_command(&mut self, command: &str, prompt: &mut Prompt) -> CommandResult {
        let input = command.trim();
        if input.is_empty() || !input.starts_with(':') {
            return CommandResult::Error("Not a command".to_string());
        }

        let parts: Vec<&str> = input[1..].splitn(2, ' ').collect();
        let cmd = parts[0];
        let arg = parts.get(1).unwrap_or(&"");

        match cmd {
            "o" => {
                if arg.is_empty() {
                    CommandResult::Error("Filename required".to_string())
                } else {
                    match std::fs::read_to_string(arg) {
                        Ok(contents) => {
                            prompt.editor.insert_str(&contents);
                            CommandResult::Success(format!("Loaded file: {}", arg))
                        }
                        Err(e) => CommandResult::Error(format!("Failed to load file: {}", e)),
                    }
                }
            }
            "w" => { // write response text to file
                if arg.is_empty() {
                    CommandResult::Error("Filename required".to_string())
                } else {
                    // Get the last AI response from the chat history
                    let last_response = self.plain_chat.iter()
                        .filter(|msg| msg.starts_with("🤖:"))
                        .last()
                        .unwrap_or(&String::new())
                        .clone();
                        
                    match std::fs::write(arg, last_response) {
                        Ok(_) => CommandResult::Success(format!("Written to file: {}", arg)),
                        Err(e) => CommandResult::Error(format!("Failed to write to file: {}", e)),
                    }
                }
            }
            "clear" => {
                self.plain_chat.clear();
                self.formatted_chat = Text::raw("");
                self.scroll = 0;
                CommandResult::Success("Chat cleared".to_string())
            }
            "save" => {
                if arg.is_empty() {
                    CommandResult::Error("Filename required".to_string())
                } else if let Err(e) = std::fs::write(arg, self.plain_chat.join("\n")) {
                    CommandResult::Error(format!("Failed to save: {}", e))
                } else {
                    CommandResult::Success(format!("Saved to {}", arg))
                }
            }
            "quit" => {
                std::process::exit(0);
            }
            _ => CommandResult::Error(format!("Unknown command: {}", cmd)),
        }
    }



    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let mut text = self.formatted_chat.clone();
        text.extend(self.answer.formatted_answer.clone());

        self.area_height = area.height;
        self.area_width = area.width;

        let scroll: u16 = {
            if self
                .automatic_scroll
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                let scroll = self.height().saturating_sub(self.area_height.into()) as u16;
                self.scroll = scroll;
                scroll
            } else {
                self.scroll
            }
        };

        let chat = Paragraph::new(text)
            .scroll((scroll, 0))
            .wrap(Wrap { trim: false })
            .block(Block::default());

        frame.render_widget(chat, area);
    }
}
