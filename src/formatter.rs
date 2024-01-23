use ansi_to_tui::IntoText;

use bat::{assets::HighlightingAssets, config::Config, controller::Controller, Input};
use ratatui::text::Text;

pub struct Formatter<'a> {
    controller: Controller<'a>,
}

impl<'a> Formatter<'a> {
    pub fn new(config: &'a Config, assets: &'a HighlightingAssets) -> Self {
        let controller = Controller::new(config, assets);
        Self { controller }
    }

    pub fn format(&self, input: &str) -> Text<'static> {
        let mut buffer = String::new();
        let input = Input::from_bytes(input.as_bytes()).name("text.md");
        self.controller
            .run(vec![input.into()], Some(&mut buffer))
            .unwrap();
        buffer.into_text().unwrap_or(Text::from(buffer))
    }
}
