mod command_viewer;
pub use command_viewer::{CommandViewer, CommandViewerState};

pub fn format_arguments(arguments: Vec<String>) -> Vec<String> {
    let mut previous_arg_collapsed = false;

    arguments.into_iter().fold(vec![], |mut buf, next| {
        let last_arg = buf.last_mut();

        if let Some(last_arg) = last_arg {
            if !next.starts_with('-') && last_arg.starts_with('-') && !previous_arg_collapsed {
                last_arg.push(' ');
                last_arg.push_str(&next);
                previous_arg_collapsed = true;
                return buf
            }
        }

        previous_arg_collapsed = false;
        buf.push(next);
        buf
    })
}
