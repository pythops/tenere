use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::{env, io};
use tenere::app::{App, AppResult};
use tenere::config::Config;
use tenere::event::{Event, EventHandler};
use tenere::formatter::Formatter;
use tenere::handler::handle_key_events;
use tenere::llm::{LLMAnswer, LLMRole};
use tenere::tui::Tui;

use tenere::llm::LLMModel;

use std::sync::Arc;
use tokio::sync::Mutex;

use clap::{crate_description, crate_version, Command};

#[tokio::main]
async fn main() -> AppResult<()> {
    Command::new("tenere")
        .about(crate_description!())
        .version(crate_version!())
        .get_matches();

    let config = Arc::new(Config::load());

    let (formatter_config, formatter_assets) = Formatter::init();
    let formatter = Formatter::new(&formatter_config, &formatter_assets);

    let mut app = App::new(config.clone(), &formatter);

    let llm = Arc::new(Mutex::new(
        LLMModel::init(&config.llm, config.clone()).await,
    ));

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => {
                handle_key_events(key_event, &mut app, llm.clone(), tui.events.sender.clone())
                    .await?;
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::LLMEvent(LLMAnswer::Answer(answer)) => {
                app.chat
                    .handle_answer(LLMAnswer::Answer(answer), &formatter);
            }
            Event::LLMEvent(LLMAnswer::EndAnswer) => {
                {
                    let mut llm = llm.lock().await;
                    llm.append_chat_msg(app.chat.answer.plain_answer.clone(), LLMRole::ASSISTANT);
                }

                app.chat.handle_answer(LLMAnswer::EndAnswer, &formatter);
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
