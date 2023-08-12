use std::collections::HashMap;
use std::{env, io};
use tenere::app::{App, AppResult};
use tenere::cli;
use tenere::config::Config;
use tenere::event::{Event, EventHandler};
use tenere::handler::handle_key_events;
use tenere::llm::LLMAnswer;
use tenere::tui::Tui;
use tui::backend::CrosstermBackend;
use tui::Terminal;

use tenere::llm::{LLMBackend, LLMModel};

use std::sync::Arc;

use clap::crate_version;

fn main() -> AppResult<()> {
    cli::cli().version(crate_version!()).get_matches();

    let config = Arc::new(Config::load());
    let mut app = App::new(config.clone());
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
                app.answer.push_str(answer.as_str());
            }
            Event::LLMEvent(LLMAnswer::EndAnswer) => {
                let mut conv: HashMap<String, String> = HashMap::new();
                conv.insert("role".to_string(), "user".to_string());
                conv.insert("content".to_string(), app.answer.clone());
                app.llm_messages.push(conv);
                app.chat.push(app.answer.clone());
                app.chat.push("\n".to_string());
                app.answer.clear();
            }
            Event::LLMEvent(LLMAnswer::StartAnswer) => {
                app.spinner.active = false;
                app.chat.pop();
                app.chat.push("🤖: ".to_string());
            }
            Event::Notification(notification) => {
                app.notifications.push(notification);
            }
        }
    }

    tui.exit()?;
    Ok(())
}
