use crate::{
    app::{App, AppResult, FocusedBlock, Mode},
    event::Event,
};

use crate::llm::LLM;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::mpsc::Sender;
use std::{collections::HashMap, thread};

use crate::notification::{Notification, NotificationLevel};
use std::sync::Arc;

pub fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App,
    llm: Arc<impl LLM + 'static>,
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
                let user_input: String = app.prompt.drain(3..).collect();
                let user_input = user_input.trim();
                if user_input.is_empty() {
                    return Ok(());
                }
                app.chat.push(format!(" : {user_input}\n"));

                let conv = HashMap::from([
                    ("role".into(), "user".into()),
                    ("content".into(), user_input.into()),
                ]);
                app.llm_messages.push(conv);

                let llm_messages = app.llm_messages.clone();

                thread::spawn(move || {
                    let response = llm.ask(llm_messages.to_vec());
                    sender
                        .send(Event::LLMAnswer(match response {
                            Ok(answer) => answer,
                            Err(e) => e.to_string(),
                        }))
                        .unwrap();
                });
                app.spinner.active = true;
                app.chat.push("🤖: Waiting ..".to_string());
            }

            // scroll down
            KeyCode::Char('j') | KeyCode::Down => match app.focused_block {
                FocusedBlock::History => {
                    if app.history_thread_index < app.history.len() - 1 {
                        app.history_thread_index += 1;
                    }
                }
                _ => app.scroll += 1,
            },

            // scroll up
            KeyCode::Char('k') | KeyCode::Up => match app.focused_block {
                FocusedBlock::History => {
                    if app.history_thread_index > 0 {
                        app.history_thread_index -= 1;
                    }
                }
                _ => app.scroll -= 1,
            },

            // Clear the prompt
            KeyCode::Char('d') => {
                if app.previous_key == KeyCode::Char('d') {
                    app.prompt = String::from(">_ ");
                }
            }

            // New chat
            KeyCode::Char(c) if c == app.config.key_bindings.new_chat => {
                app.prompt = String::from(">_ ");
                app.history.push(app.chat.clone());
                app.chat = Vec::new();
                app.llm_messages = Vec::new();
                app.scroll = 0;
            }

            // Save chat
            KeyCode::Char(c) if c == app.config.key_bindings.save_chat => match app.focused_block {
                FocusedBlock::History | FocusedBlock::Preview => {
                    if !app.history.is_empty() {
                        let chat: String = app.history[app.history_thread_index]
                            .iter()
                            .map(|m| m.to_string())
                            .collect();
                        match std::fs::write(app.config.archive_file_name.clone(), chat) {
                            Ok(_) => {
                                let notif = Notification::new(
                                    format!(
                                        "**Info**\nChat saved to `{}` file",
                                        app.config.archive_file_name
                                    ),
                                    NotificationLevel::Info,
                                );

                                sender.send(Event::Notification(notif)).unwrap();
                            }
                            Err(e) => {
                                let notif = Notification::new(
                                    format!("**Error**\n{e}"),
                                    NotificationLevel::Error,
                                );

                                sender.send(Event::Notification(notif)).unwrap();
                            }
                        }
                    }
                }
                FocusedBlock::Chat | FocusedBlock::Prompt => {
                    let chat: String = app.chat.iter().map(|m| m.to_string()).collect();
                    match std::fs::write(app.config.archive_file_name.clone(), chat) {
                        Ok(_) => {
                            let notif = Notification::new(
                                format!(
                                    "**Info**\nChat saved to `{}` file",
                                    app.config.archive_file_name
                                ),
                                NotificationLevel::Info,
                            );

                            sender.send(Event::Notification(notif)).unwrap();
                        }
                        Err(e) => {
                            let notif = Notification::new(
                                format!("**Error**\n{e}"),
                                NotificationLevel::Error,
                            );

                            sender.send(Event::Notification(notif)).unwrap();
                        }
                    }
                }
            },

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
                    app.scroll = 0;
                }
            }

            // kill the app
            KeyCode::Char('c' | 'C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.running = false;
                }
            }

            // Show help
            KeyCode::Char(c) if c == app.config.key_bindings.show_help => {
                app.show_help_popup = true;
            }

            // Show history
            KeyCode::Char(c) if c == app.config.key_bindings.show_history => {
                app.show_history_popup = true;
                app.focused_block = FocusedBlock::History;
            }

            // Discard help & history popups
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
