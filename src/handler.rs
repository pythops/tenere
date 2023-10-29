use crate::app::{Chat, Prompt};
use crate::llm::LLMAnswer;

use crate::{
    app::{App, AppResult, FocusedBlock, Mode},
    event::Event,
};
use colored::*;

use crate::llm::LLM;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::mpsc::Sender;
use std::{collections::HashMap, thread};

use tui::text::Line;

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

            // Terminate the stream response
            KeyCode::Char('t') => {
                app.terminate_response_signal
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }

            // Submit the prompt
            KeyCode::Enter => {
                let user_input: String = app.prompt.message.drain(3..).collect();
                let user_input = user_input.trim();

                if user_input.is_empty() {
                    return Ok(());
                }

                app.chat.messages.push(format!(" : {}\n", user_input));

                app.chat
                    .formatted_chat
                    .lines
                    .push(Line::raw(format!(" : {}\n", user_input)));

                let conv = HashMap::from([
                    ("role".into(), "user".into()),
                    ("content".into(), user_input.into()),
                ]);
                app.llm_messages.push(conv);

                let llm_messages = app.llm_messages.clone();

                app.spinner.active = true;

                app.chat
                    .formatted_chat
                    .lines
                    .push(Line::raw("🤖: ".to_string()));

                let terminate_response_signal = app.terminate_response_signal.clone();

                thread::spawn(move || {
                    let res = llm.ask(llm_messages.to_vec(), &sender, terminate_response_signal);
                    if let Err(e) = res {
                        sender
                            .send(Event::LLMEvent(LLMAnswer::StartAnswer))
                            .unwrap();
                        sender
                            .send(Event::LLMEvent(LLMAnswer::Answer(
                                e.to_string().red().to_string(),
                            )))
                            .unwrap();
                    }
                });
            }

            // scroll down
            KeyCode::Char('j') | KeyCode::Down => match app.focused_block {
                FocusedBlock::History => {
                    if app.history.index < app.history.chat.len() - 1 {
                        app.history.index += 1;
                    }
                }
                _ => {
                    app.scroll = app.scroll.saturating_add(1);
                }
            },

            // scroll up
            KeyCode::Char('k') | KeyCode::Up => match app.focused_block {
                FocusedBlock::History => {
                    if app.history.index > 0 {
                        app.history.index -= 1;
                    }
                }
                _ => {
                    app.scroll = app.scroll.saturating_sub(1);
                }
            },

            // Clear the prompt
            KeyCode::Char('d') => {
                if app.previous_key == KeyCode::Char('d') {
                    app.prompt = Prompt::default();
                }
            }

            // New chat
            KeyCode::Char(c) if c == app.config.key_bindings.new_chat => {
                app.prompt = Prompt::default();
                app.history
                    .formatted_chat
                    .push(app.chat.formatted_chat.clone());
                app.history.chat.push(app.chat.messages.clone());
                app.chat = Chat::default();
                app.llm_messages = Vec::new();
                app.scroll = 0;
            }

            // Save chat
            KeyCode::Char(c) if c == app.config.key_bindings.save_chat => match app.focused_block {
                FocusedBlock::History | FocusedBlock::Preview => {
                    if !app.history.chat.is_empty() {
                        match std::fs::write(
                            &app.config.archive_file_name,
                            app.history.chat[app.history.index].join(""),
                        ) {
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
                                    format!("**Error**\n{}", e),
                                    NotificationLevel::Error,
                                );

                                sender.send(Event::Notification(notif)).unwrap();
                            }
                        }
                    }
                }
                FocusedBlock::Chat | FocusedBlock::Prompt => {
                    match std::fs::write(
                        app.config.archive_file_name.clone(),
                        app.chat.messages.join(""),
                    ) {
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
                                format!("**Error**\n{}", e),
                                NotificationLevel::Error,
                            );

                            sender.send(Event::Notification(notif)).unwrap();
                        }
                    }
                }
                _ => (),
            },

            // Switch the focus
            KeyCode::Tab => match app.focused_block {
                FocusedBlock::Chat => {
                    app.scroll = 0;
                    app.focused_block = FocusedBlock::Prompt;
                }
                FocusedBlock::Prompt => {
                    app.scroll =
                        app.chat.formatted_chat.height() + app.answer.formatted_answer.height();
                    app.focused_block = FocusedBlock::Chat;
                }

                FocusedBlock::History => {
                    app.scroll = 0;
                    app.focused_block = FocusedBlock::Preview
                }
                FocusedBlock::Preview => {
                    app.scroll = 0;
                    app.focused_block = FocusedBlock::History
                }
                FocusedBlock::Help => (),
            },

            // kill the app
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.running = false;
                }
            }

            // Show help
            KeyCode::Char(c) if c == app.config.key_bindings.show_help => {
                app.focused_block = FocusedBlock::Help;
            }

            // Show history
            KeyCode::Char(c) if c == app.config.key_bindings.show_history => {
                app.focused_block = FocusedBlock::History;
            }

            // Discard help & history popups
            KeyCode::Esc => {
                app.focused_block = FocusedBlock::Prompt;
            }

            _ => {}
        },

        Mode::Insert => match key_event.code {
            KeyCode::Enter => app.prompt.message.push('\n'),

            KeyCode::Char(c) => {
                app.prompt.message.push(c);
            }

            KeyCode::Backspace => {
                if app.prompt.message.len() > 3 {
                    app.prompt.message.pop();
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
