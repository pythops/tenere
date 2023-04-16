use crate::{
    app::{App, AppResult, FocusedBlock, Mode},
    event::Event,
    gpt::GPT,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::mpsc::Sender;
use std::{collections::HashMap, thread};

pub fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App,
    gpt: &GPT,
    sender: Sender<Event>,
) -> AppResult<()> {
    match app.mode {
        Mode::Normal => match key_event.code {
            // Change mode to Insert
            KeyCode::Char('i') => {
                app.mode = Mode::Insert;
                app.focused_block = FocusedBlock::Prompt;
            }

            // Quit the app
            KeyCode::Char('q') => {
                app.running = false;
            }

            // TODO: handle shift + enter. Limitation from crossterm
            KeyCode::Enter => {
                let mut conv: HashMap<String, String> = HashMap::new();

                let user_input: String = app.input.drain(3..).collect();
                let user_input = user_input.trim();
                if user_input.is_empty() {
                    return Ok(());
                }
                app.messages.push(format!(" : {}\n", user_input));

                conv.insert("role".to_string(), "user".to_string());
                conv.insert("content".to_string(), user_input.to_string());
                app.history.push(conv.clone());

                let history = app.history.clone();
                let gpt = gpt.clone();
                thread::spawn(move || {
                    let response = gpt.ask(history);
                    sender
                        .send(Event::GPTResponse(match response {
                            Ok(answer) => answer,
                            Err(e) => e.to_string(),
                        }))
                        .unwrap();
                });
                app.messages.push("🤖: Waiting ...".to_string());
            }

            // scroll down
            KeyCode::Char('j') => {
                app.scroll += 1;
            }

            KeyCode::Down => {
                app.scroll += 1;
            }

            // scroll up
            KeyCode::Char('k') => {
                app.scroll -= 1;
            }

            KeyCode::Up => {
                app.scroll -= 1;
            }

            // Clear the prompt
            KeyCode::Char('d') => {
                if app.previous_key == KeyCode::Char('d') {
                    app.input = String::from(">_ ");
                }
            }

            // Clear the prompt and the chat
            KeyCode::Char('l') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.input = String::from(">_ ");
                    app.messages = Vec::new();
                    app.history = Vec::new();
                    app.scroll = 0;
                }
            }

            // Switch the focus
            KeyCode::Tab => {
                match app.focused_block {
                    FocusedBlock::Chat => app.focused_block = FocusedBlock::Prompt,
                    FocusedBlock::Prompt => app.focused_block = FocusedBlock::Chat,
                }
                app.scroll = 0
            }

            // kill the app
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.running = false;
                }
            }

            // Help popup
            KeyCode::Char('h') => {
                app.show_help_popup = true;
            }

            // Discard help popup
            KeyCode::Esc => {
                if app.show_help_popup {
                    app.show_help_popup = false;
                }
            }

            _ => {}
        },

        Mode::Insert => match key_event.code {
            // New line
            KeyCode::Enter => app.input.push('\n'),

            KeyCode::Char(c) => {
                app.input.push(c);
            }

            // Remove char from the prompt
            KeyCode::Backspace => {
                if app.input.len() > 3 {
                    app.input.pop();
                }
            }

            //Switch to Normal mode
            KeyCode::Esc => {
                app.mode = Mode::Normal;
            }
            _ => {}
        },
    }

    app.previous_key = key_event.code;
    Ok(())
}
