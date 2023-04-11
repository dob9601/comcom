use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version)]
pub struct Cli {
    #[clap(trailing_var_arg = true)]
    pub command: Vec<String>
}
