use crate::app::{App, AppResult, FocusedBlock, Mode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
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

                let assisstant_message = match app.gpt.ask(app.history.clone()).await {
                    Ok(answer) => {
                        conv.insert("role".to_string(), "user".to_string());
                        conv.insert("content".to_string(), answer.clone());
                        answer
                    }
                    Err(_) => "Error".to_string(),
                };

                app.messages.push(format!("🤖: {}\n", assisstant_message));

                app.messages.push("\n".to_string());
                app.history.push(conv);
            }

            // scroll down
            KeyCode::Char('j') => {
                app.scroll += 1;
            }

            // scroll up
            KeyCode::Char('k') => {
                if app.scroll > 0 {
                    app.scroll -= 1;
                }
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
