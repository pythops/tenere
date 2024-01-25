use std::{rc::Rc, sync::atomic::AtomicBool};

use ratatui::{
    layout::Rect,
    style::Style,
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::{app::FocusedBlock, formatter::Formatter, llm::LLMAnswer};

#[derive(Debug, Clone, Default)]
pub struct Answer<'a> {
    pub plain_answer: String,
    pub formatted_answer: Text<'a>,
}

#[derive(Debug, Clone)]
pub struct Chat<'a> {
    pub plain_chat: Vec<String>,
    pub formatted_chat: Text<'a>,
    pub answer: Answer<'a>,
    pub scroll: u16,
    area_height: u16,
    area_width: u16,
    pub automatic_scroll: Rc<AtomicBool>,
    pub block: Block<'a>,
}

impl Default for Chat<'_> {
    fn default() -> Self {
        let block = Block::default()
            .border_type(BorderType::default())
            .borders(Borders::ALL)
            .style(Style::default());

        Self {
            plain_chat: Vec::new(),
            formatted_chat: Text::raw(""),
            answer: Answer::default(),
            scroll: 0,
            area_height: 0,
            area_width: 0,
            automatic_scroll: Rc::new(AtomicBool::new(true)),
            block,
        }
    }
}

impl Chat<'_> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_answer(&mut self, event: LLMAnswer, formatter: &Formatter) {
        match event {
            LLMAnswer::StartAnswer => {
                self.formatted_chat.lines.pop();
            }

            LLMAnswer::Answer(answer) => {
                self.answer.plain_answer.push_str(answer.as_str());

                self.answer.formatted_answer =
                    formatter.format(format!("🤖: {}", &self.answer.plain_answer).as_str());
            }

            LLMAnswer::EndAnswer => {
                self.formatted_chat
                    .extend(self.answer.formatted_answer.clone());

                self.formatted_chat.extend(Text::raw("\n"));

                self.plain_chat
                    .push(format!("🤖: {}", self.answer.plain_answer));

                self.answer = Answer::default();
            }
        }
    }

    pub fn height(&self) -> usize {
        let mut chat = self.formatted_chat.clone();

        chat.extend(self.answer.formatted_answer.clone());

        let nb_lines = chat.lines.len() + 3;
        chat.lines.iter().fold(nb_lines, |acc, line| {
            acc + line.width() / self.area_width as usize
        })
    }

    pub fn move_to_bottom(&mut self) {
        self.scroll = (self.formatted_chat.height() + self.answer.formatted_answer.height())
            .saturating_sub((self.area_height - 2).into()) as u16;
    }

    pub fn move_to_top(&mut self) {
        self.scroll = 0;
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused_block: &FocusedBlock) {
        let mut text = self.formatted_chat.clone();
        text.extend(self.answer.formatted_answer.clone());

        self.area_height = area.height;
        self.area_width = area.width;

        let scroll: u16 = {
            if self
                .automatic_scroll
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                let scroll = self.height().saturating_sub(self.area_height.into()) as u16;
                self.scroll = scroll;
                scroll
            } else {
                self.scroll
            }
        };

        let chat = Paragraph::new(text)
            .scroll((scroll, 0))
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default())
                    .border_type(match focused_block {
                        FocusedBlock::Chat => BorderType::Thick,
                        _ => BorderType::Rounded,
                    })
                    .border_style(Style::default()),
            );

        frame.render_widget(chat, area);
    }
}
