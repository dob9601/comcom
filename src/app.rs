use std::error::Error;
use std::io::{Read, Stdout, Write};
use std::{io, process};

use clipboard::{ClipboardContext, ClipboardProvider};
use colored::Colorize;
use comcom::components::{CommandViewer, Component, Documentation, EditorMode};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::text::Spans;
use tui::widgets::Paragraph;
use tui::{Frame, Terminal};

pub struct App {
    command_viewer: CommandViewer,
    documentation: Documentation,
}

impl App {
    pub fn new(command_name: String, command_arguments: Vec<String>) -> Self {
        Self {
            documentation: Documentation::new(&command_name),
            command_viewer: CommandViewer::new(command_name, command_arguments),
        }
    }

    fn generate_ui<B: Backend>(&mut self, frame: &mut Frame<B>) -> Result<(), Box<dyn Error>> {
        let outer = Layout::default()
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(frame.size());

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(60), Constraint::Min(0)].as_ref())
            .split(outer[0]);

        let taskbar = Spans::from(vec![self.command_viewer.mode.into()]);
        let instructions = Paragraph::new(taskbar); //"<A-Enter> = Run | <i> = Insert Mode | <ESC> = Normal Mode | <j> = Down | <k> = Up");
        frame.render_widget(instructions, outer[1]);

        // let command_block = Block::default().title("┤Command├").borders(Borders::ALL);
        // let command_viewer = CommandViewer::new(
        //     self.command_name.clone(),
        //     self.command_arguments.clone(),
        //     self.selected_index,
        //     self.mode == AppMode::Insert,
        // )
        // .block(command_block);
        //
        // frame.render_stateful_widget(
        //     command_viewer,
        //     chunks[0],
        //     &mut CommandViewerState::new(self.selected_index, self.mode == AppMode::Insert),
        // );

        self.command_viewer.draw(frame, &chunks[0])?;
        self.documentation.draw(frame, &chunks[1])?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            SetTitle(format!("comcom: {}", &self.command_viewer.name))
        )?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| self.generate_ui(f).unwrap())?;
            let event = event::read()?;

            self.documentation.handle_event(&event)?;
            self.command_viewer.handle_event(&event)?;

            if self.command_viewer.mode == EditorMode::Normal {
                if let Event::Key(key) = event {
                    match (key.code, key.modifiers) {
                        (KeyCode::Enter, KeyModifiers::ALT) => {
                            self.execute_command(&mut terminal).unwrap();
                            terminal.draw(|f| self.generate_ui(f).unwrap())?;
                        }
                        (KeyCode::Char('q'), KeyModifiers::NONE) => break,
                        (KeyCode::Char('E'), KeyModifiers::NONE) => break,
                        (KeyCode::Char('y'), KeyModifiers::NONE) => {
                            let mut ctx: ClipboardContext = ClipboardProvider::new()?;
                            ctx.set_contents(
                                self.command_viewer.name.clone()
                                    + " "
                                    + &self.command_viewer.arguments.join(" "),
                            )
                            .unwrap();
                        }
                        _ => {}
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }

    fn execute_command(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), Box<dyn Error>> {
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        process::Command::new(self.command_viewer.name.clone())
            .args(self.command_viewer.arguments.clone())
            .spawn()?
            .wait()?;

        print!("{}", "Press 'ENTER' to return to comcom".blue().bold());
        io::stdout().flush()?;
        let _ = io::stdin().read(&mut [0u8]).unwrap();

        enable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            EnterAlternateScreen,
            EnableMouseCapture,
            SetTitle(format!("comcom: {}", self.command_viewer.name.clone())),
        )?;

        // Trigger a redraw
        terminal.resize(Rect::default())?;

        Ok(())
    }
}
