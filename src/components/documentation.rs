use std::cmp::{max, min};
use std::process::Command;

use crossterm::event::{Event, KeyCode, KeyModifiers};
use tui::backend::Backend;
use tui::layout::Rect;
use tui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

use crate::ComcomError;

use super::Component;

pub struct Documentation {
    offset: usize,
    text: String,
}

impl Documentation {
    pub fn new(command: &str) -> Self {
        let command_output = &Command::new("man")
            .args(["-P", "cat"])
            .arg(command)
            .output()
            .unwrap()
            .stdout;

        let mut text = String::from_utf8_lossy(command_output).to_string();
        if text.is_empty() {
            text = "No Documentation Available".to_string()
        }

        Self { offset: 0, text }
    }
}

impl Component for Documentation {
    fn draw<B: Backend>(
        &mut self,
        frame: &mut tui::Frame<B>,
        area: &Rect,
    ) -> Result<(), ComcomError> {
        let documentation_block = Block::default()
            .title("┤Documentation├")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let documentation = Paragraph::new(self.text.splitn(self.offset + 1, '\n').last().unwrap())
            .wrap(Wrap { trim: true })
            .block(documentation_block);
        frame.render_widget(documentation, *area);
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> Result<(), ComcomError> {
        if let Event::Key(key) = event {
            match (key.code, key.modifiers) {
                (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                    if self.offset + 5 < self.text.matches('\n').count() {
                        self.offset += 5;
                    }
                }
                (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                    self.offset -= min(self.offset, 5);
                }
                _ => {}
            }
        };
        Ok(())
    }
}
