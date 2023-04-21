use std::collections::HashMap;
use std::{env, io};
use tenere::app::{App, AppResult};
use tenere::cli;
use tenere::event::{Event, EventHandler};
use tenere::gpt::GPT;
use tenere::handler::handle_key_events;
use tenere::tui::Tui;
use tui::backend::CrosstermBackend;
use tui::Terminal;

use clap::crate_version;

fn main() -> AppResult<()> {
    cli::cli().version(crate_version!()).get_matches();

    let mut app = App::new();
    let gpt = GPT::new(app.config.gpt.openai_api_key.clone());

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
                handle_key_events(key_event, &mut app, &gpt, tui.events.sender.clone())?
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::GPTResponse(response) => {
                app.chat.pop();
                app.chat.push(format!("🤖: {}\n", response));
                app.chat.push("\n".to_string());
                let mut conv: HashMap<String, String> = HashMap::new();
                conv.insert("role".to_string(), "user".to_string());
                conv.insert("content".to_string(), response.clone());
                app.gpt_messages.push(conv);
            }
        }
    }

    tui.exit()?;
    Ok(())
}
