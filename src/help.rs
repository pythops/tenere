use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders, Clear, Padding, Row, Table, TableState},
    Frame,
};

pub struct Help {
    block_height: usize,
    state: TableState,
    keys: &'static [(&'static str, &'static str)],
}

impl Default for Help {
    fn default() -> Self {
        let mut state = TableState::new().with_offset(0);
        state.select(Some(0));

        Self {
            block_height: 0,
            state,
            keys: &[
                ("Esc", "Switch to Normal mode"),
                ("i", "Switch to Insert mode"),
                ("v", "Switch to Visual mode"),
                ("G", "Go to the end"),
                ("gg", "Go to the top"),
                (
                    "ctrl + n",
                    "Start new chat and save the previous one to the history",
                ),
                (
                    "ctrl + s",
                    "Save the chat to  file in the current directory",
                ),
                ("Tab", "Switch the focus"),
                ("ctrl + h", "Show history"),
                ("ctrl + t", "Stop the stream response"),
                ("j or Down", "Scroll down"),
                ("k or Up", "Scroll up"),
                ("?", "show help"),
            ],
        }
    }
}

impl Help {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scroll_down(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.keys.len().saturating_sub(self.block_height - 4) {
                    i
                } else {
                    i + 1
                }
            }
            None => 1,
        };
        *self.state.offset_mut() = i;
        self.state.select(Some(i));
    }
    pub fn scroll_up(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i > 1 {
                    i - 1
                } else {
                    0
                }
            }
            None => 1,
        };
        *self.state.offset_mut() = i;
        self.state.select(Some(i));
    }

    pub fn render(&mut self, frame: &mut Frame, block: Rect) {
        self.block_height = block.height as usize;
        let widths = [Constraint::Length(15), Constraint::Min(60)];
        let rows: Vec<Row> = self
            .keys
            .iter()
            .map(|key| Row::new(vec![key.0, key.1]))
            .collect();

        let table = Table::new(rows, widths).block(
            Block::default()
                .padding(Padding::uniform(1))
                .title(" Help ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .style(Style::default())
                .border_type(BorderType::Thick)
                .border_style(Style::default()),
        );

        frame.render_widget(Clear, block);
        frame.render_stateful_widget(table, block, &mut self.state);
    }
}
