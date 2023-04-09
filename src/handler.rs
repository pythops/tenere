use crate::app::{App, AppResult, FocusedBlock, Mode};
// #[allow(unused_imports)]
// use crate::gpt;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match app.mode {
        Mode::Normal => match key_event.code {
            KeyCode::Char('i') => {
                app.mode = Mode::Insert;
                app.focused_block = FocusedBlock::Prompt;
            }

            KeyCode::Char('q') => {
                app.running = false;
            }

            // TODO: handle shift + enter. Limitation from crossterm
            KeyCode::Enter => {
                let user_input: String = app.input.drain(3..).collect();
                let user_input = user_input.trim();
                if user_input.is_empty() {
                    return Ok(());
                }

                // let assisstant_message = gpt::ask_gpt(&user_input).await?;

                app.messages.push(format!(" : {}\n", user_input));
                app.messages.push(format!("🪄: {}\n", "hello ".repeat(100)));
                // app.messages.push(format!("🪄: hellow \n world\n"));
                // app.messages.push(format!("🪄: {}", assisstant_message));
                app.messages.push("\n".to_string());

                app.scroll = 0;
            }

            // scroll down
            KeyCode::Char('j') => {
                app.scroll += 1;
            }

            // scroll up
            KeyCode::Char('k') => {
                app.scroll -= 1;
            }

            KeyCode::Char('d') => {
                if app.previous_key == KeyCode::Char('d') {
                    app.input = String::from(">_ ");
                    app.scroll = 0;
                }
            }
            KeyCode::Char('l') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.input = String::from(">_ ");
                    app.messages = Vec::new();
                    app.scroll = 0;
                }
            }

            KeyCode::Tab => match app.focused_block {
                FocusedBlock::Chat => app.focused_block = FocusedBlock::Prompt,
                FocusedBlock::Prompt => app.focused_block = FocusedBlock::Chat,
            },

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

    app.previous_key = key_event.code;
    Ok(())
}
