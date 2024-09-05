use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, Cell, Clear, Padding, Row, Table},
    Frame,
};

pub struct Help {
    block_height: usize,
    keys: Vec<(Cell<'static>, &'static str)>,
}

impl Default for Help {
    fn default() -> Self {
        Self {
            block_height: 0,
            keys: vec![
                (
                    Cell::from("Esc").bold().yellow(),
                    "Switch to Normal mode / Dismiss pop-up",
                ),
                (Cell::from("Tab").bold().yellow(), "Switch the focus"),
                (
                    Cell::from("ctrl + n").bold().yellow(),
                    "Start new chat and save the previous one to the history",
                ),
                (
                    Cell::from("ctrl + s").bold().yellow(),
                    "Save the chat to file in the current directory",
                ),
                (Cell::from("ctrl + h").bold().yellow(), "Show history"),
                (
                    Cell::from("ctrl + t").bold().yellow(),
                    "Stop the stream response",
                ),
                (Cell::from("j or Down").bold().yellow(), "Scroll down"),
                (Cell::from("k or Up").bold().yellow(), "Scroll up"),
                (Cell::from("G").bold().yellow(), "Go to the end"),
                (Cell::from("gg").bold().yellow(), "Go to the top"),
                (Cell::from("?").bold().yellow(), "Show help"),
            ],
        }
    }
}

impl Help {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(15),
                Constraint::Fill(1),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(frame.area());

        let block = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Min(75),
                Constraint::Fill(1),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(layout[1])[1];

        self.block_height = block.height as usize;
        let widths = [Constraint::Length(12), Constraint::Fill(1)];
        let rows: Vec<Row> = self
            .keys
            .iter()
            .map(|key| {
                Row::new(vec![key.0.to_owned(), key.1.into()])
                    .style(Style::default().fg(Color::White))
            })
            .collect();

        let table = Table::new(rows, widths).block(
            Block::default()
                .padding(Padding::uniform(1))
                .title(" Help ")
                .title_style(Style::default().bold().fg(Color::Green))
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .style(Style::default())
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(Color::Green)),
        );

        frame.render_widget(Clear, block);
        frame.render_widget(table, block);
    }
}
