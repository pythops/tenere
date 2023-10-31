use crate::notification::NotificationLevel;
use std;

use crate::app::{App, FocusedBlock, Mode};
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
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
    // Layout
    let frame_size = frame.size();

    // prompt height can grow till 40% of the frame height
    let prompt_block_max_height = (0.4 * frame_size.height as f32) as u16;

    let prompt_content_height = {
        let mut height: u16 = 1;
        for line in app.prompt.message.lines() {
            height += 1;
            height += line.width() as u16 / frame_size.width;
        }
        height
    };

    let prompt_block_height = std::cmp::min(prompt_content_height, prompt_block_max_height);

    // chat height is the frame height minus the prompt height
    let chat_block_height = std::cmp::max(
        frame_size.height - prompt_block_height - 3,
        frame_size.height - prompt_block_max_height - 3,
    );

    let (chat_block, prompt_block) = {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(chat_block_height),
                    Constraint::Length(prompt_block_height),
                ]
                .as_ref(),
            )
            .split(frame.size());
        (chunks[0], chunks[1])
    };

    // prompt block
    let prompt_paragraph = {
        if let FocusedBlock::Prompt = app.focused_block {
            let diff: isize = prompt_content_height as isize - prompt_block_max_height as isize - 1;

            // case where the prompt content height is shorter than the prompt block height
            if diff < 0 {
                app.prompt.scroll = 0;
            } else {
                let diff = diff as u16;
                if app.prompt.scroll > diff {
                    app.prompt.scroll = diff
                }
            }
        }

        Paragraph::new(app.prompt.message.as_str())
            .wrap(Wrap { trim: false })
            .scroll((app.prompt.scroll, 0))
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

        Mode::Insert => frame.set_cursor(
            prompt_block.x
                + {
                    let last_line = app.prompt.message.lines().last().unwrap_or("");
                    let mut width = last_line.len() as u16;
                    if last_line.len() as u16 > frame_size.width {
                        let last_word = last_line.rsplit(' ').last().unwrap_or("");
                        width =
                            last_line.width() as u16 % frame_size.width + last_word.len() as u16;
                    }
                    width
                }
                + 1,
            prompt_block.y + std::cmp::min(prompt_content_height, prompt_block_max_height) - 1,
        ),
    }

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
                        FocusedBlock::Chat => match app.mode {
                            Mode::Insert => Style::default().fg(Color::Green),
                            Mode::Normal => Style::default().fg(Color::Yellow),
                        },
                        _ => Style::default(),
                    }),
            )
    };

    // Draw

    frame.render_widget(chat_paragraph, chat_block);

    frame.render_widget(prompt_paragraph, prompt_block);

    if let FocusedBlock::History | FocusedBlock::Preview = app.focused_block {
        let area = centered_rect(80, 80, frame_size);

        let (history_block, preview_block) = {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(area);
            (chunks[0], chunks[1])
        };

        let history = List::new({
            if app.history.chat.is_empty() {
                vec![ListItem::new(Line::raw("History is empty"))]
            } else {
                app.history
                    .formatted_chat
                    .iter()
                    .enumerate()
                    .map(|(i, chat)| {
                        let msg = chat.lines[0].clone();
                        ListItem::new(msg).style({
                            if app.history.index == i {
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

        let preview_chat: Text = if !app.history.chat.is_empty() {
            app.history.formatted_chat[app.history.index].clone()
        } else {
            Text::from("")
        };

        let preview_scroll = {
            if !app.history.formatted_chat.is_empty() {
                let preview_chat_height = app.history.formatted_chat[app.history.index].lines.len();

                let diff = preview_chat_height as i32 - preview_block.height as i32 + 3;

                if let FocusedBlock::Preview = app.focused_block {
                    if diff >= 0 {
                        let diff = diff as u16;

                        if app.history.scroll >= diff {
                            app.history.scroll = diff;
                        }
                    }
                }
            }
            app.history.scroll
        };

        let preview = Paragraph::new(preview_chat)
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

    if let FocusedBlock::Help = app.focused_block {
        let help = format!(
            "
`i`            : Switch to Insert mode
`Esc`          : Switch to Normal mode
`dd`           : Clear the prompt
`G`            : Go to the end
`gg`           : Go to the top
`n`            : Start new chat and save the previous one to the history
`s`            : Save the chat to `{}` file in the current directory
`Tab`          : Switch the focus
`h`            : Show history
`t`            : Stop the stream response
`j` or `Down`  : Scroll down
`k` or `Up`    : Scroll up
`?`            : show help
`q`            : Quit
",
            app.config.archive_file_name
        );

        let block = Paragraph::new(help.as_str())
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
        let area = help_rect(frame_size);
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
    }

    for (i, n) in app.notifications.iter().enumerate() {
        let border_color = match n.level {
            NotificationLevel::Info => Color::Green,
            NotificationLevel::Warning => Color::Yellow,
            NotificationLevel::Error => Color::Red,
        };

        let block = Paragraph::new(if !n.message.is_empty() {
            Text::from(n.message.as_str())
        } else {
            Text::from("")
        })
        .wrap(Wrap { trim: false })
        .alignment(tui::layout::Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default())
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color)),
        );
        let area = notification_rect(i as u16, frame_size);
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
    }
}
