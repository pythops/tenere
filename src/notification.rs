use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub ttl: u16,
}

#[derive(Debug, Clone)]
pub enum NotificationLevel {
    Error,
    Warning,
    Info,
}

impl Notification {
    pub fn new(message: String, level: NotificationLevel) -> Self {
        Self {
            message,
            level,
            ttl: 8,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, block: Rect) {
        let (color, title) = match self.level {
            NotificationLevel::Info => (Color::Green, "Info"),
            NotificationLevel::Warning => (Color::Yellow, "Warning"),
            NotificationLevel::Error => (Color::Red, "Error"),
        };

        let text = Text::from(vec![
            Line::styled(
                title,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center),
            Line::raw(self.message.as_str()),
        ]);

        let para = Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default())
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(color)),
            );

        frame.render_widget(Clear, block);
        frame.render_widget(para, block);
    }
}
