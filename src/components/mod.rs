use crossterm::event::Event;
use tui::Frame;
use tui::backend::Backend;
use tui::layout::Rect;

mod documentation;
pub use documentation::Documentation;

mod command_viewer;
pub use command_viewer::{CommandViewer, EditorMode};

use crate::ComcomError;

pub trait Component {
    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, area: &Rect) -> Result<(), ComcomError>;

    fn handle_event(&mut self, _event: &Event) -> Result<(), ComcomError> {
        Ok(())
    }
}
