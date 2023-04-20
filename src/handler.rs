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

            KeyCode::Enter => {
                let mut conv: HashMap<String, String> = HashMap::new();

                let user_input: String = app.prompt.drain(3..).collect();
                let user_input = user_input.trim();
                if user_input.is_empty() {
                    return Ok(());
                }
                app.chat.push(format!(" : {}\n", user_input));

                conv.insert("role".to_string(), "user".to_string());
                conv.insert("content".to_string(), user_input.to_string());
                app.gpt_messages.push(conv.clone());

                let gpt_messages = app.gpt_messages.clone();
                let gpt = gpt.clone();
                thread::spawn(move || {
                    let response = gpt.ask(gpt_messages);
                    sender
                        .send(Event::GPTResponse(match response {
                            Ok(answer) => answer,
                            Err(e) => e.to_string(),
                        }))
                        .unwrap();
                });
                app.chat.push("🤖: Waiting ...".to_string());
            }

            // scroll down
            KeyCode::Char('j') => match app.focused_block {
                FocusedBlock::History => {
                    if app.history_thread_index < app.history.len() - 1 {
                        app.history_thread_index += 1;
                    }
                }
                _ => app.scroll += 1,
            },

            KeyCode::Down => {
                app.scroll += 1;
            }

            // scroll up
            KeyCode::Char('k') => match app.focused_block {
                FocusedBlock::History => {
                    if app.history_thread_index > 0 {
                        app.history_thread_index -= 1;
                    }
                }
                _ => app.scroll -= 1,
            },

            KeyCode::Up => {
                app.scroll -= 1;
            }

            // Clear the prompt
            KeyCode::Char('d') => {
                if app.previous_key == KeyCode::Char('d') {
                    app.prompt = String::from(">_ ");
                }
            }

            // Clear the prompt and the chat
            KeyCode::Char('l') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.prompt = String::from(">_ ");
                    app.history.push(app.chat.clone());
                    app.chat = Vec::new();
                    app.gpt_messages = Vec::new();
                    app.scroll = 0;
                }
            }

            // Switch the focus
            KeyCode::Tab => {
                if app.show_history_popup {
                    match app.focused_block {
                        FocusedBlock::Preview => app.focused_block = FocusedBlock::History,
                        FocusedBlock::History => app.focused_block = FocusedBlock::Preview,
                        _ => (),
                    }
                } else {
                    match app.focused_block {
                        FocusedBlock::Chat => app.focused_block = FocusedBlock::Prompt,
                        FocusedBlock::Prompt => app.focused_block = FocusedBlock::Chat,
                        _ => (),
                    }
                    app.scroll = 0
                }
            }

            // kill the app
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.running = false;
                }
            }

            // Help popup
            KeyCode::Char('?') => {
                app.show_help_popup = true;
            }

            // Help popup
            KeyCode::Char('h') => {
                app.show_history_popup = true;
                app.focused_block = FocusedBlock::History;
            }

            // Discard help popup
            KeyCode::Esc => {
                app.show_help_popup = false;
                if app.show_history_popup {
                    app.show_history_popup = false;
                    app.focused_block = FocusedBlock::Prompt;
                    app.scroll = 0;
                }
            }

            _ => {}
        },

        Mode::Insert => match key_event.code {
            // New line
            KeyCode::Enter => app.prompt.push('\n'),

            KeyCode::Char(c) => {
                app.prompt.push(c);
            }

            // Remove char from the prompt
            KeyCode::Backspace => {
                if app.prompt.len() > 3 {
                    app.prompt.pop();
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
