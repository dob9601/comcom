use std::error::Error;
use std::io::{Read, Stdout, Write};
use std::{io, process};

use clipboard::{ClipboardContext, ClipboardProvider};
use colored::Colorize;
use comcom::{CommandViewer, CommandViewerState};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::{Frame, Terminal};

#[derive(PartialEq, Clone)]
enum AppMode {
    Normal,
    Insert,
}

impl From<AppMode> for Span<'_> {
    fn from(val: AppMode) -> Self {
        match val {
            AppMode::Normal => {
                Span::styled("-NORMAL-", Style::default().fg(Color::Black).bg(Color::Red))
            }
            AppMode::Insert => Span::styled(
                "-INSERT-",
                Style::default().fg(Color::Black).bg(Color::Green),
            ),
        }
    }
}

pub struct App {
    command_name: String,
    command_arguments: Vec<String>,
    selected_index: usize,
    mode: AppMode,
    documentation: String,
    documentation_offset: u32,
}

impl App {
    pub fn new(command_name: String, command_arguments: Vec<String>) -> Self {
        let command_output = &process::Command::new("man")
            .args(["-P", "cat"])
            .arg(&command_name)
            .output()
            .unwrap()
            .stdout;

        let documentation = String::from_utf8_lossy(command_output).to_string();
        Self {
            command_name,
            command_arguments,
            mode: AppMode::Normal,
            selected_index: 0,
            documentation,
            documentation_offset: 0,
        }
    }

    fn generate_ui<B: Backend>(&mut self, frame: &mut Frame<B>) {
        let outer = Layout::default()
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(frame.size());

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(60), Constraint::Min(0)].as_ref())
            .split(outer[0]);

        let taskbar = Spans::from(vec![self.mode.clone().into()]);
        let instructions = Paragraph::new(taskbar); //"<A-Enter> = Run | <i> = Insert Mode | <ESC> = Normal Mode | <j> = Down | <k> = Up");
        frame.render_widget(instructions, outer[1]);

        let command_block = Block::default().title("┤Command├").borders(Borders::ALL);
        let command_viewer = CommandViewer::new(
            self.command_name.clone(),
            self.command_arguments.clone(),
            self.selected_index,
            self.mode == AppMode::Insert,
        )
        .block(command_block);

        frame.render_stateful_widget(
            command_viewer,
            chunks[0],
            &mut CommandViewerState::new(self.selected_index, self.mode == AppMode::Insert),
        );

        let documentation_block = Block::default()
            .title("┤Documentation├")
            .borders(Borders::ALL);

        let documentation = Paragraph::new(
            self.documentation
                .splitn(self.documentation_offset as usize + 1, '\n')
                .last()
                .unwrap(),
        )
        .wrap(Wrap { trim: true })
        .block(documentation_block);
        frame.render_widget(documentation, chunks[1]);
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            SetTitle(format!("comcom: {}", self.command_name.clone()))
        )?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| self.generate_ui(f))?;

            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers, &self.mode) {
                    (KeyCode::Char(c), KeyModifiers::NONE, AppMode::Insert) => {
                        self.command_arguments[self.selected_index - 1].push(c)
                    }
                    (KeyCode::Backspace, KeyModifiers::NONE, AppMode::Insert) => {
                        let argument = &mut self.command_arguments[self.selected_index - 1];
                        if !argument.is_empty() {
                            argument.remove(argument.len() - 1);
                        }
                    }
                    (KeyCode::Esc | KeyCode::Enter, _, AppMode::Insert) => {
                        if self.command_arguments[self.selected_index - 1].is_empty() {
                            self.command_arguments.remove(self.selected_index - 1);
                        }
                        self.mode = AppMode::Normal
                    }
                    (KeyCode::Enter, KeyModifiers::ALT, _) => {
                        self.execute_command(&mut terminal).unwrap();
                        terminal.draw(|f| self.generate_ui(f))?;
                    }
                    (KeyCode::Enter, KeyModifiers::NONE, AppMode::Normal) => {
                        if self.selected_index > 0 {
                            self.mode = AppMode::Insert
                        }
                    }
                    (KeyCode::Char('q'), _, _) => break,
                    (KeyCode::Char('E'), KeyModifiers::NONE, _) => break,
                    (KeyCode::Char('o'), _, _) => {
                        self.command_arguments
                            .insert(self.selected_index, "".to_string());
                        self.selected_index += 1;
                        self.mode = AppMode::Insert
                    }
                    (KeyCode::Char('y'), _, _) => {
                        let mut ctx: ClipboardContext = ClipboardProvider::new()?;
                        ctx.set_contents(
                            self.command_name.clone() + " " + &self.command_arguments.join(" "),
                        )
                        .unwrap();
                    }
                    (KeyCode::Char('O'), _, _) => {
                        if self.selected_index > 0 {
                            self.command_arguments
                                .insert(self.selected_index - 1, "".to_string());
                            self.mode = AppMode::Insert
                        }
                    }
                    (KeyCode::Down | KeyCode::Char('j'), _, _) => {
                        if self.selected_index < self.command_arguments.len() {
                            self.selected_index += 1
                        } else {
                            self.selected_index = 0
                        }
                    }
                    (KeyCode::Up | KeyCode::Char('k'), _, _) => {
                        if self.selected_index != 0 {
                            self.selected_index -= 1
                        } else {
                            self.selected_index = self.command_arguments.len()
                        }
                    }
                    (KeyCode::Char('d'), KeyModifiers::CONTROL, _) => {
                        if (self.documentation_offset as usize) < self.documentation.matches('\n').count() - 5 {
                            self.documentation_offset += 5;
                        }
                    }
                    (KeyCode::Char('u'), KeyModifiers::CONTROL, _) => {
                        if self.documentation_offset > 0 {
                            self.documentation_offset -= 5;
                        }
                    }
                    _ => {}
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
        process::Command::new(self.command_name.clone())
            .args(self.command_arguments.clone())
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
            SetTitle(format!("comcom: {}", self.command_name.clone())),
        )?;

        // Trigger a redraw
        terminal.resize(Rect::default())?;

        Ok(())
    }
}
