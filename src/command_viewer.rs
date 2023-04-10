use tui::style::{Modifier, Style};
use tui::widgets::{Block, List, ListItem, ListState, StatefulWidget};

#[derive(Default, Debug)]
pub struct CommandViewerState {
    pub selected_index: usize,
    pub editing: bool,
}

impl CommandViewerState {
    pub fn new(selected_index: usize, editing: bool) -> Self {
        Self {
            selected_index,
            editing,
        }
    }
}

pub struct CommandViewer<'a> {
    command: List<'a>,
}

impl<'a> CommandViewer<'a> {
    pub fn new(
        command_name: String,
        command_arguments: Vec<String>,
        selected_index: usize,
        edit_mode: bool
    ) -> Self {
        let mut command_with_args = vec![ListItem::new(command_name)];
        command_with_args.extend(
            command_arguments
                .into_iter()
                .enumerate()
                .map(|(index, arg)| {
                    let mut formatted_arg = format!("    {arg}");

                    if index + 1 == selected_index && edit_mode {
                        formatted_arg.push('_')
                    }

                    ListItem::new(formatted_arg)
                }),
        );

        Self {
            command: List::new(command_with_args)
                .highlight_symbol("> ")
                .highlight_style(Style::default().add_modifier(Modifier::BOLD)),
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.command = self.command.block(block);
        self
    }
}

impl StatefulWidget for CommandViewer<'_> {
    type State = CommandViewerState;

    fn render(
        self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: &mut Self::State,
    ) {
        let mut list_state = ListState::default();
        list_state.select(Some(state.selected_index));

        self.command.render(area, buf, &mut list_state);
    }
}
