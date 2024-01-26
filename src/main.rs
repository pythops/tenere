use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::collections::HashMap;
use std::{env, io};
use tenere::app::{App, AppResult};
use tenere::cli;
use tenere::config::Config;
use tenere::event::{Event, EventHandler};
use tenere::formatter::Formatter;
use tenere::handler::handle_key_events;
use tenere::llm::LLMAnswer;
use tenere::tui::Tui;

use tenere::llm::LLMModel;

use std::sync::Arc;

use clap::crate_version;

fn main() -> AppResult<()> {
    cli::cli().version(crate_version!()).get_matches();

    let config = Arc::new(Config::load());

    let (formatter_config, formatter_assets) = Formatter::init();
    let formatter = Formatter::new(&formatter_config, &formatter_assets);

    let mut app = App::new(config.clone(), &formatter);

    let llm = Arc::new(LLMModel::init(&config.model, config.clone()));

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
                app.chat
                    .handle_answer(LLMAnswer::Answer(answer), &formatter);
            }
            Event::LLMEvent(LLMAnswer::EndAnswer) => {
                app.chat.handle_answer(LLMAnswer::EndAnswer, &formatter);

                // TODO: factor this into llm struct or trait
                let mut conv: HashMap<String, String> = HashMap::new();
                conv.insert("role".to_string(), "user".to_string());
                conv.insert("content".to_string(), app.chat.answer.plain_answer.clone());
                app.llm_messages.push(conv);

                app.terminate_response_signal
                    .store(false, std::sync::atomic::Ordering::Relaxed);
            }
            Event::LLMEvent(LLMAnswer::StartAnswer) => {
                app.spinner.active = false;
                app.chat.handle_answer(LLMAnswer::StartAnswer, &formatter);
            }

            Event::Notification(notification) => {
                app.notifications.push(notification);
            }
        }
    }

    tui.exit()?;
    Ok(())
}
