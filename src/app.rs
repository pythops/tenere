use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub enum Mode {
    Normal,
    Insert,
}
#[derive(Debug)]
pub struct App {
    pub input: String,
    pub mode: Mode,
    pub running: bool,
    pub messages: Vec<String>,
    pub input_window_height: u16,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            input: String::from(">_ "),
            input_window_height: 2,
            mode: Mode::Normal,
            messages: Vec::new(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&self) {}

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        // Layout
        let app_area = frame.size();

        let vmid_area = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(app_area)[0];

        let vh_mid_area = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(vmid_area)[0];

        let container = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Yellow));

        let inside_container = container.inner(vh_mid_area);

        // TODO: set the max for the chunks[1]
        let (assisstant_block, user_block, mode_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints(
                    [
                        Constraint::Max(80),
                        Constraint::Length(self.input.width() as u16 / app_area.width + 1),
                        Constraint::Length(2),
                    ]
                    .as_ref(),
                )
                .split(inside_container);
            (chunks[0], chunks[1], chunks[2])
        };

        // end layout

        let input = Paragraph::new(self.input.as_ref())
            .style(match self.mode {
                Mode::Normal => Style::default(),
                Mode::Insert => Style::default().fg(Color::Yellow),
            })
            .wrap(Wrap { trim: false })
            .block(Block::default());

        match self.mode {
            Mode::Normal => {}

            // TODO: set the cursor position
            Mode::Insert => {
                frame.set_cursor(user_block.x + self.input.width() as u16, user_block.y)
            }
        }

        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .map(|m| {
                //TODO: make the ListItem shows in multiple lines
                let content = Spans::from(Span::raw(m));
                ListItem::new(content)
            })
            .collect();

        let messages = List::new(messages).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_type(BorderType::Rounded),
        );

        let mode = Paragraph::new({
            match self.mode {
                Mode::Normal => "Mode: Normal",
                Mode::Insert => "Mode: Insert",
            }
        })
        .block(Block::default().borders(Borders::TOP));

        // Draw
        frame.render_widget(container, vh_mid_area);
        frame.render_widget(messages, assisstant_block);
        frame.render_widget(input, user_block);
        frame.render_widget(mode, mode_block);
    }
}
