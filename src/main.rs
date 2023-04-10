use std::{env, io};
use tenere::app::{App, AppResult};
use tenere::event::{Event, EventHandler};
use tenere::handler::handle_key_events;
use tenere::tui::Tui;
use tui::backend::CrosstermBackend;
use tui::Terminal;

#[tokio::main]
async fn main() -> AppResult<()> {
    match env::var("OPENAI_API_KEY") {
        Ok(_) => {}
        Err(_) => {
            eprintln!("OPENAI_API_KEY environment variable is not set");
            std::process::exit(1);
        }
    }
    let mut app = App::new();

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app).await?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    tui.exit()?;
    Ok(())
}
