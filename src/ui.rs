use crate::notification::NotificationLevel;
use ansi_to_tui::IntoText;
use std;

use crate::app::{App, FocusedBlock, Mode};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

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
                Constraint::Length(15),
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

pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    // Layout
    let app_area = frame.size();

    // prompt height can grow till 40% of the frame height
    let max_prompt_height = (0.4 * app_area.height as f32) as u16;
    let prompt_height = {
        let mut height: u16 = 1;
        for line in app.prompt.lines() {
            height += 1;
            height += line.width() as u16 / app_area.width;
        }
        height
    };

    // chat height is the frame height minus the prompt height
    let max_chat_height = app_area.height - max_prompt_height - 3;
    let chat_height = app_area.height - prompt_height - 3;

    let (chat_block, prompt_block) = {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(std::cmp::max(chat_height, max_chat_height)),
                    Constraint::Length(std::cmp::min(prompt_height, max_prompt_height)),
                    // Constraint::Length(2),
                ]
                .as_ref(),
            )
            .split(frame.size());
        (chunks[0], chunks[1])
    };

    // prompt block
    //TODO: show scroll bar
    let prompt = {
        let mut scroll = 0;
        let height_diff = prompt_height as i32 - max_prompt_height as i32;
        if let FocusedBlock::Prompt = app.focused_block {
            if height_diff + app.scroll >= 0 {
                scroll = height_diff + app.scroll;
            }

            // scroll up case
            if height_diff > 0 && -app.scroll > height_diff {
                app.scroll = -height_diff;
            }

            // Scroll down case
            // 2 empty lines
            if height_diff > 0 && app.scroll >= 2 {
                app.scroll = 2
            }
        }
        Paragraph::new(app.prompt.as_str())
            .wrap(Wrap { trim: false })
            .scroll((scroll as u16, 0))
            .style(Style::default())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default())
                    .border_type(match app.focused_block {
                        FocusedBlock::Prompt => BorderType::Thick,
                        _ => BorderType::Rounded,
                    })
                    .border_style(match app.focused_block {
                        FocusedBlock::Prompt => match app.mode {
                            Mode::Insert => Style::default().fg(Color::Green),
                            Mode::Normal => Style::default().fg(Color::Yellow),
                        },
                        _ => Style::default(),
                    }),
            )
    };

    match app.mode {
        Mode::Normal => {}

        // TODO: set the cursor position
        Mode::Insert => frame.set_cursor(
            prompt_block.x
                + {
                    let last_line = app.prompt.lines().last().unwrap_or("");
                    let mut width = last_line.len() as u16;
                    if last_line.len() as u16 > app_area.width {
                        let last_word = last_line.rsplit(' ').last().unwrap_or("");
                        width = last_line.width() as u16 % app_area.width + last_word.len() as u16;
                    }
                    width
                }
                + 1,
            prompt_block.y + std::cmp::min(prompt_height, max_prompt_height) - 1,
        ),
    }

    // Chat block
    let chat = {
        let messages: String = app.chat.iter().map(|m| m.to_string()).collect();

        let messages_height = {
            let mut height: u16 = 0;
            for msg in &app.chat {
                height += 1;
                for line in msg.lines() {
                    height += 1;
                    height += line.width() as u16 / app_area.width;
                }
            }
            height
        };

        let mut scroll = 0;
        let height_diff = messages_height as i32
            - std::cmp::max(chat_height, max_chat_height) as i32
            - app.chat.len() as i32
            + 1;
        if height_diff > 0 {
            scroll = height_diff;
        }
        if let FocusedBlock::Chat = app.focused_block {
            if height_diff + app.scroll >= 0 {
                scroll = height_diff + app.scroll;
            }

            // // scroll up case
            if height_diff > 0 && -app.scroll > height_diff {
                app.scroll = -height_diff;
            }
            //
            // // Scroll down case
            if height_diff > 0 && app.scroll > 0 {
                app.scroll = 0;
            }
        }

        Paragraph::new({
            termimad::term_text(messages.as_str())
                .to_string()
                .into_text()
                .unwrap_or(Text::from(messages))
        })
        .scroll((scroll as u16, 0))
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
                    FocusedBlock::Chat => match app.mode {
                        Mode::Insert => Style::default().fg(Color::Green),
                        Mode::Normal => Style::default().fg(Color::Yellow),
                    },
                    _ => Style::default(),
                }),
        )
    };

    // Draw
    frame.render_widget(chat, chat_block);
    frame.render_widget(prompt, prompt_block);

    if app.show_history_popup {
        let area = centered_rect(80, 80, app_area);

        let (history_block, preview_block) = {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(area);
            (chunks[0], chunks[1])
        };

        let history = List::new({
            if app.history.is_empty() {
                vec![ListItem::new(Line::from(Span::from("History is empty")))]
            } else {
                app.history
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        let msg = c[0].clone().strip_prefix(" : ").unwrap().to_string();
                        let content = Line::from(Span::from(msg));
                        ListItem::new(content).style({
                            if app.history_thread_index == i {
                                Style::default().bg(Color::Rgb(50, 54, 26))
                            } else {
                                Style::default()
                            }
                        })
                    })
                    .collect::<Vec<ListItem>>()
            }
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" History ")
                .title_alignment(tui::layout::Alignment::Center)
                .style(Style::default())
                .border_type(BorderType::Rounded)
                .border_style(match app.focused_block {
                    FocusedBlock::History => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }),
        );

        let preview_chat: String = if !app.history.is_empty() {
            app.history[app.history_thread_index]
                .iter()
                .map(|m| m.to_string())
                .collect()
        } else {
            String::new()
        };

        let preview_scroll = {
            let mut height: u16 = 0;
            let mut scroll: u16 = 0;
            for line in preview_chat.lines() {
                height += 1;
                height += line.width() as u16 / preview_block.width;
            }

            let height_diff = height as i32 - preview_block.height as i32;

            if height_diff > 0 {
                if let FocusedBlock::Preview = app.focused_block {
                    if app.scroll < 0 {
                        app.scroll = 0;
                        scroll = app.scroll as u16;
                    }
                    if app.scroll > height_diff {
                        app.scroll = height_diff;
                        scroll = app.scroll as u16;
                    }
                    if app.scroll >= 0 {
                        scroll = app.scroll as u16;
                    }
                }
            }
            scroll
        };

        let preview = Paragraph::new({
            termimad::term_text(preview_chat.as_str())
                .to_string()
                .into_text()
                .unwrap_or(Text::from(preview_chat))
        })
        .wrap(Wrap { trim: false })
        .scroll((preview_scroll, 0))
        .block(
            Block::default()
                .title(" Preview ")
                .title_alignment(tui::layout::Alignment::Center)
                .borders(Borders::ALL)
                .style(Style::default())
                .border_type(BorderType::Rounded)
                .border_style(match app.focused_block {
                    FocusedBlock::Preview => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }),
        );

        frame.render_widget(Clear, area);
        frame.render_widget(history, history_block);
        frame.render_widget(preview, preview_block);
    }

    if app.show_help_popup {
        let help = format!(
            "
`i`          : Switch to Insert mode
`Esc`        : Switch to Normal mode
`dd`         : Clear the prompt
`n`          : Start new chat and save the previous one to the history
`s`          : Save the chat to `{}` file in the current directory
`Tab`        : Switch the focus
`h`          : Show history
`j` or `Down`  : Scroll down
`k` or `Up`    : Scroll up
`?`          : show help
`q`          : Quit
",
            app.config.archive_file_name
        );

        let block = Paragraph::new(
            termimad::term_text(help.as_str())
                .to_string()
                .into_text()
                .unwrap_or(Text::from(help)),
        )
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .title(" Help ")
                .title_alignment(tui::layout::Alignment::Center)
                .borders(Borders::ALL)
                .style(Style::default())
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow)),
        );
        let area = help_rect(app_area);
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
    }

    for (i, n) in app.notifications.iter().enumerate() {
        let border_color = match n.level {
            NotificationLevel::Info => Color::Green,
            NotificationLevel::Warning => Color::Yellow,
            NotificationLevel::Error => Color::Red,
        };

        let block = Paragraph::new(
            termimad::term_text(n.message.clone().as_str())
                .to_string()
                .into_text()
                .unwrap_or(Text::from(n.message.clone())),
        )
        .wrap(Wrap { trim: false })
        .alignment(tui::layout::Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default())
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color)),
        );
        let area = notification_rect(i as u16, app_area);
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
    }
}
