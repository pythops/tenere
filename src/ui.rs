use std;

use crate::app::{App, FocusedBlock};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn render(app: &mut App, frame: &mut Frame) {
    let frame_size = frame.area();

    let prompt_block_height = app.prompt.height(&frame_size) + 3;

    let (chat_block, prompt_block) = {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(prompt_block_height)].as_ref())
            .split(frame.area());
        (chunks[0], chunks[1])
    };

    // Chat
    app.chat.render(frame, chat_block);

    // Prompt
    app.prompt.render(frame, prompt_block, &app.focused_block);

    // History
    if let FocusedBlock::History | FocusedBlock::Preview = app.focused_block {
        app.history.render(frame, &app.focused_block);
    }

    // Help
    if let FocusedBlock::Help = app.focused_block {
        app.help.render(frame);
    }

    // Notifications
    for (index, notification) in app.notifications.iter().enumerate() {
        notification.render(index, frame);
    }
}
