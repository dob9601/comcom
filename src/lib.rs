use clap::Parser;
use thiserror::Error;

use self::cli::Cli;

pub mod components;
pub mod cli;

#[derive(Error, Debug)]
pub enum ComcomError {}

pub fn format_arguments(arguments: Vec<String>) -> Vec<String> {
    let arguments_iter = arguments
        .into_iter()
        .flat_map(|args| args.split(' ').into_iter().map(String::from).collect::<Vec<String>>());

    let mut previous_arg_collapsed = false;

    arguments_iter.fold(vec![], |mut buf, next| {
        let last_arg = buf.last_mut();

        if let Some(last_arg) = last_arg {
            if !next.starts_with('-') && last_arg.starts_with('-') && !previous_arg_collapsed {
                last_arg.push(' ');
                last_arg.push_str(&next);
                previous_arg_collapsed = true;
                return buf;
            }
        }

        previous_arg_collapsed = false;
        buf.push(next.to_string());
        buf
    })
}
