use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::collections::HashMap;
use std::{env, io};
use tenere::app::{Answer, App, AppResult};
use tenere::cli;
use tenere::config::Config;
use tenere::event::{Event, EventHandler};
use tenere::formatter::Formatter;
use tenere::handler::handle_key_events;
use tenere::llm::LLMAnswer;
use tenere::tui::Tui;

use ratatui::text::Text;
use tenere::llm::{LLMBackend, LLMModel};

use std::sync::Arc;

use clap::crate_version;

fn main() -> AppResult<()> {
    cli::cli().version(crate_version!()).get_matches();

    let config = Arc::new(Config::load());

    // TODO: move this to init app
    // Text formatter
    let formatter_config = bat::config::Config {
        colored_output: true,
        ..Default::default()
    };
    let formatter_assets = bat::assets::HighlightingAssets::from_binary();

    let formatter = Formatter::new(&formatter_config, &formatter_assets);

    let mut app = App::new(config.clone(), &formatter);

    let llm = Arc::new(LLMModel::init(LLMBackend::ChatGPT, config));

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => {
                handle_key_events(key_event, &mut app, llm.clone(), tui.events.sender.clone())?
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::LLMEvent(LLMAnswer::Answer(answer)) => {
                if app.answer.answer.is_empty() {
                    app.answer
                        .answer
                        .push_str(format!("🤖: {}", answer).as_str());
                }
                app.answer.answer.push_str(answer.as_str());
                app.answer.formatted_answer = formatter.format(&app.answer.answer);
            }
            Event::LLMEvent(LLMAnswer::EndAnswer) => {
                app.answer.answer = app
                    .answer
                    .answer
                    .strip_prefix("🤖: ")
                    .unwrap_or_default()
                    .to_string();

                // TODO: factor this into llm struct or trait
                let mut conv: HashMap<String, String> = HashMap::new();
                conv.insert("role".to_string(), "user".to_string());
                conv.insert("content".to_string(), app.answer.answer.clone());
                app.llm_messages.push(conv);

                app.chat.formatted_chat.extend(app.answer.formatted_answer);
                app.chat.formatted_chat.extend(Text::raw("\n"));

                app.chat.messages.push(format!("🤖: {}", app.answer.answer));

                app.answer = Answer::default();

                app.terminate_response_signal
                    .store(false, std::sync::atomic::Ordering::Relaxed);
            }
            Event::LLMEvent(LLMAnswer::StartAnswer) => {
                app.spinner.active = false;
                app.chat.formatted_chat.lines.pop();
            }
            Event::Notification(notification) => {
                app.notifications.push(notification);
            }
        }
    }

    tui.exit()?;
    Ok(())
}
