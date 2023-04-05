use crate::app::{App, AppResult, Mode};
// #[allow(unused_imports)]
// use crate::gpt;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match app.mode {
        Mode::Normal => match key_event.code {
            KeyCode::Char('i') => {
                app.mode = Mode::Insert;
            }

            KeyCode::Esc | KeyCode::Char('q') => {
                app.running = false;
            }

            // TODO: handle shift + enter. Limitation from crossterm
            KeyCode::Enter => {
                let user_input: String = app.input.drain(3..).collect();

                // let assisstant_message = gpt::ask_gpt(&user_input).await?;

                app.messages.push(format!(" : {}", user_input));
                app.messages.push(format!("🪄: {}", "hello ".repeat(100)));
                // app.messages.push(format!("🪄: {}", assisstant_message));
                app.messages.push("\n".to_string());
            }

            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.running = false;
                }
            }

            _ => {}
        },

        Mode::Insert => match key_event.code {
            KeyCode::Enter => app.input.push('\n'),

            KeyCode::Char(c) => {
                app.input.push(c);
            }
            KeyCode::Backspace => {
                if app.input.len() > 3 {
                    app.input.pop();
                }
            }
            KeyCode::Esc => {
                app.mode = Mode::Normal;
            }
            _ => {}
        },
    }
    Ok(())
}
