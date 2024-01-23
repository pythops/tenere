use std;

use crate::app::{App, FocusedBlock};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn notification_rect(offset: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1 + 5 * offset),
                Constraint::Length(5),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(74),
                Constraint::Percentage(25),
                Constraint::Percentage(1),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn help_rect(r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(22),
                Constraint::Length(18),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length((r.width - 85) / 2),
                Constraint::Length(85),
                Constraint::Length((r.width - 85) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn render(app: &mut App, frame: &mut Frame) {
    let frame_size = frame.size();

    let prompt_block_height = app.prompt.height(&frame_size) + 3;
    let chat_block_height = frame_size.height - prompt_block_height;

    let (chat_block, prompt_block) = {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(prompt_block_height)].as_ref())
            .split(frame.size());
        (chunks[0], chunks[1])
    };

    // Chat block
    let chat_text = {
        let mut c = app.chat.formatted_chat.clone();
        c.extend(app.answer.formatted_answer.clone());
        c
    };

    let chat_messages_height = {
        let nb_lines = chat_text.lines.len() + 3;
        let messages_height = chat_text.lines.iter().fold(nb_lines, |acc, line| {
            acc + line.width() / frame_size.width as usize
        });

        messages_height
    };

    let chat_paragraph = {
        let diff: isize = chat_messages_height as isize - chat_block_height as isize;

        if let FocusedBlock::Chat = app.focused_block {
            if diff > 0 {
                let diff = diff as u16;
                app.chat.length = diff;

                if app.chat.scroll >= diff {
                    app.chat.scroll = diff;
                }
            }
        } else {
            app.chat.scroll = if diff > 0 { diff as u16 } else { 0 };
        }

        Paragraph::new(chat_text)
            .scroll((app.chat.scroll, 0))
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default())
                    .border_type(match app.focused_block {
                        FocusedBlock::Chat => BorderType::Thick,
                        _ => BorderType::Rounded,
                    })
                    .border_style(match app.focused_block {
                        FocusedBlock::Chat => Style::default(),
                        _ => Style::default(),
                    }),
            )
    };

    // Render

    // Prompt
    frame.render_widget(chat_paragraph, chat_block);
    app.prompt.render(frame, prompt_block);

    // History
    if let FocusedBlock::History | FocusedBlock::Preview = app.focused_block {
        let area = centered_rect(80, 80, frame_size);
        app.history.render(frame, area, app.focused_block.clone());
    }

    // Help
    if let FocusedBlock::Help = app.focused_block {
        app.prompt.update(&FocusedBlock::Help);
        let area = help_rect(frame_size);
        app.help.render(frame, area);
    }

    // Notifications
    for (i, notif) in app.notifications.iter_mut().enumerate() {
        let area = notification_rect(i as u16, frame_size);
        notif.render(frame, area);
    }
}
