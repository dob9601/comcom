use std::env;
use std::error::Error;

mod app;
use app::App;
use clap::Parser;
use comcom::cli::Cli;
use comcom::format_arguments;

fn main() -> Result<(), Box<dyn Error>> {
    let Cli { command: arguments } = Cli::parse();
    let mut args_iter = arguments.into_iter();

    let command_name: String = args_iter.next().unwrap();
    let command_arguments: Vec<String> = format_arguments(args_iter.collect());

    let mut app = App::new(command_name, command_arguments);
    app.run()
}
