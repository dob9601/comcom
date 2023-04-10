use std::env;
use std::error::Error;

mod app;
use app::App;
use comcom::format_arguments;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args_iter = env::args().skip(1);

    let command_name: String = args_iter.next().unwrap();
    let command_arguments: Vec<String> = format_arguments(args_iter.collect());

    let mut app = App::new(command_name, command_arguments);
    app.run()
}
