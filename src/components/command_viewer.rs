use crossterm::event::{Event, KeyCode, KeyModifiers};
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, BorderType, Borders, List, ListItem, ListState};
use tui::Frame;

use crate::{ComcomError, format_arguments};

use super::Component;

#[derive(PartialEq, Clone, Copy)]
pub enum EditorMode {
    Normal,
    Insert,
}

impl From<EditorMode> for Span<'_> {
    fn from(val: EditorMode) -> Self {
        match val {
            EditorMode::Normal => {
                Span::styled("-NORMAL-", Style::default().fg(Color::Black).bg(Color::Red))
            }
            EditorMode::Insert => Span::styled(
                "-INSERT-",
                Style::default().fg(Color::Black).bg(Color::Green),
            ),
        }
    }
}

pub struct CommandViewer {
    pub name: String,
    pub arguments: Vec<String>,
    pub selected_index: usize,
    pub mode: EditorMode,
}

impl CommandViewer {
    pub fn new(name: String, arguments: Vec<String>) -> Self {
        Self {
            name,
            arguments,
            selected_index: 0,
            mode: EditorMode::Normal,
        }
    }
}

impl Component for CommandViewer {
    fn draw<B: Backend>(
        &mut self,
        frame: &mut Frame<B>,
        area: &Rect,
    ) -> Result<(), crate::ComcomError> {
        let mut command_with_args = vec![ListItem::new(&*self.name)];
        command_with_args.extend(
            self.arguments
                .iter()
                .cloned()
                .enumerate()
                .map(|(index, arg)| {
                    let mut formatted_arg = format!("    {arg}");

                    if index + 1 == self.selected_index && self.mode == EditorMode::Insert {
                        formatted_arg.push('_')
                    }

                    ListItem::new(formatted_arg)
                }),
        );

        let block = Block::default()
            .title("┤Command├")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let widget = List::new(command_with_args)
            .highlight_symbol("> ")
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .block(block);

        let mut state = ListState::default();
        state.select(Some(self.selected_index));

        frame.render_stateful_widget(widget, *area, &mut state);
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> Result<(), ComcomError> {
        if let Event::Key(key) = event {
            match (key.code, key.modifiers, &self.mode) {
                (KeyCode::Char(c), KeyModifiers::NONE, EditorMode::Insert) => {
                    self.arguments[self.selected_index - 1].push(c)
                }
                (KeyCode::Backspace, KeyModifiers::NONE, EditorMode::Insert) => {
                    let argument = &mut self.arguments[self.selected_index - 1];
                    if !argument.is_empty() {
                        argument.remove(argument.len() - 1);
                    }
                }
                (KeyCode::Esc | KeyCode::Enter, _, EditorMode::Insert) => {
                    if self.arguments[self.selected_index - 1].is_empty() {
                        self.arguments.remove(self.selected_index - 1);
                    }
                    let args = std::mem::take(&mut self.arguments);
                    self.arguments = format_arguments(args);
                    self.mode = EditorMode::Normal
                }
                (KeyCode::Enter, KeyModifiers::NONE, EditorMode::Normal) => {
                    if self.selected_index > 0 {
                        self.mode = EditorMode::Insert
                    }
                }
                (KeyCode::Char('o'), KeyModifiers::NONE, EditorMode::Normal) => {
                    self.arguments.insert(self.selected_index, "".to_string());
                    self.selected_index += 1;
                    self.mode = EditorMode::Insert
                }
                (KeyCode::Char('O'), KeyModifiers::NONE, EditorMode::Normal) => {
                    if self.selected_index > 0 {
                        self.arguments
                            .insert(self.selected_index - 1, "".to_string());
                        self.mode = EditorMode::Insert
                    }
                }
                (KeyCode::Down | KeyCode::Char('j'), KeyModifiers::NONE, EditorMode::Normal) => {
                    if self.selected_index < self.arguments.len() {
                        self.selected_index += 1
                    } else {
                        self.selected_index = 0
                    }
                }
                (KeyCode::Up | KeyCode::Char('k'), KeyModifiers::NONE, EditorMode::Normal) => {
                    if self.selected_index != 0 {
                        self.selected_index -= 1
                    } else {
                        self.selected_index = self.arguments.len()
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
