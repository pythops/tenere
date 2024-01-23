use crate::llm::LLMAnswer;
use crate::{app::Chat, prompt::Mode};

use crate::{
    app::{App, AppResult, FocusedBlock},
    event::Event,
};
use colored::*;

use crate::llm::LLM;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::mpsc::Sender;
use std::{collections::HashMap, thread};

use ratatui::text::Line;

use crate::notification::{Notification, NotificationLevel};
use std::sync::Arc;

pub fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App,
    llm: Arc<impl LLM + 'static>,
    sender: Sender<Event>,
) -> AppResult<()> {
    match key_event.code {
        // Quit the app
        KeyCode::Char('q') if app.prompt.mode != Mode::Insert => {
            app.running = false;
        }

        KeyCode::Char('c') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.running = false;
        }

        // Terminate the stream response
        KeyCode::Char('t') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.terminate_response_signal
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }

        // scroll down
        KeyCode::Char('j') | KeyCode::Down => match app.focused_block {
            FocusedBlock::History => {
                if !app.history.formatted_chat.is_empty()
                    && app.history.index < app.history.chat.len() - 1
                {
                    app.history.index += 1;
                }
            }

            FocusedBlock::Chat => {
                app.chat.scroll = app.chat.scroll.saturating_add(1);
            }

            FocusedBlock::Preview => {
                app.history.scroll = app.history.scroll.saturating_add(1);
            }
            FocusedBlock::Help => {
                app.help.scroll_down();
            }
            _ => (),
        },

        // scroll up
        KeyCode::Char('k') | KeyCode::Up => match app.focused_block {
            FocusedBlock::History => app.history.index = app.history.index.saturating_sub(1),

            FocusedBlock::Preview => {
                app.history.scroll = app.history.scroll.saturating_sub(1);
            }

            FocusedBlock::Chat => {
                app.chat.scroll = app.chat.scroll.saturating_sub(1);
            }

            FocusedBlock::Help => {
                app.help.scroll_up();
            }

            _ => (),
        },

        // New chat
        KeyCode::Char(c)
            if c == app.config.key_bindings.new_chat
                && key_event.modifiers == KeyModifiers::CONTROL =>
        {
            app.prompt.clear();

            app.history
                .formatted_chat
                .push(app.chat.formatted_chat.clone());
            app.history.chat.push(app.chat.messages.clone());
            app.chat = Chat::default();
            app.llm_messages = Vec::new();

            app.chat.scroll = 0;
        }

        // Save chat
        KeyCode::Char(c)
            if c == app.config.key_bindings.save_chat
                && key_event.modifiers == KeyModifiers::CONTROL =>
        {
            match app.focused_block {
                FocusedBlock::History | FocusedBlock::Preview => {
                    if !app.history.chat.is_empty() {
                        match std::fs::write(
                            &app.config.archive_file_name,
                            app.history.chat[app.history.index].join(""),
                        ) {
                            Ok(_) => {
                                let notif = Notification::new(
                                    format!(
                                        "Chat saved to `{}` file",
                                        app.config.archive_file_name
                                    ),
                                    NotificationLevel::Info,
                                );

                                sender.send(Event::Notification(notif)).unwrap();
                            }
                            Err(e) => {
                                let notif =
                                    Notification::new(e.to_string(), NotificationLevel::Error);

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
                                format!("Chat saved to `{}` file", app.config.archive_file_name),
                                NotificationLevel::Info,
                            );

                            sender.send(Event::Notification(notif)).unwrap();
                        }
                        Err(e) => {
                            let notif = Notification::new(e.to_string(), NotificationLevel::Error);

                            sender.send(Event::Notification(notif)).unwrap();
                        }
                    }
                }
                _ => (),
            }
        }

        // Switch the focus
        KeyCode::Tab => match app.focused_block {
            FocusedBlock::Chat => {
                app.focused_block = FocusedBlock::Prompt;
                app.prompt.update(&app.focused_block);
            }
            FocusedBlock::Prompt => {
                app.chat.scroll = (app.chat.formatted_chat.height()
                    + app.answer.formatted_answer.height())
                    as u16;
                app.focused_block = FocusedBlock::Chat;
                app.prompt.mode = Mode::Normal;
                app.prompt.update(&app.focused_block);
            }
            FocusedBlock::History => {
                app.focused_block = FocusedBlock::Preview;
                app.history.scroll = 0;
            }
            FocusedBlock::Preview => {
                app.focused_block = FocusedBlock::History;
                app.history.scroll = 0;
            }
            _ => (),
        },

        // Show help
        KeyCode::Char(c)
            if c == app.config.key_bindings.show_help && app.prompt.mode != Mode::Insert =>
        {
            app.focused_block = FocusedBlock::Help;
        }

        // Show history
        KeyCode::Char(c)
            if c == app.config.key_bindings.show_history
                && app.prompt.mode != Mode::Insert
                && key_event.modifiers == KeyModifiers::CONTROL =>
        {
            app.focused_block = FocusedBlock::History;
        }

        // Discard help & history popups
        KeyCode::Esc => match app.focused_block {
            FocusedBlock::History | FocusedBlock::Preview | FocusedBlock::Help => {
                app.focused_block = FocusedBlock::Prompt
            }
            _ => {}
        },

        // Go to the end: G
        KeyCode::Char('G') => match app.focused_block {
            FocusedBlock::Chat => app.chat.scroll = app.chat.length,
            FocusedBlock::History => {
                if !app.history.formatted_chat.is_empty() {
                    app.history.index = app.history.formatted_chat.len() - 1;
                }
            }
            FocusedBlock::Preview => app.history.scroll = app.history.length,
            _ => (),
        },

        _ => {}
    }

    if let FocusedBlock::Prompt = app.focused_block {
        if let Mode::Normal = app.prompt.mode {
            if key_event.code == KeyCode::Enter {
                let user_input = app.prompt.editor.lines().join("\n");
                let user_input = user_input.trim();
                if user_input.is_empty() {
                    return Ok(());
                }

                app.prompt.clear();

                app.chat.messages.push(format!(" : {}\n", user_input));

                app.chat.formatted_chat.extend(
                    app.formatter
                        .format(format!(" : {}\n", user_input).as_str()),
                );

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

                let sender = sender.clone();

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
        }

        app.prompt.handler(key_event, app.clipboard.as_mut());
    }

    Ok(())
}
