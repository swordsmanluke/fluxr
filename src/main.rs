extern crate regex;

use executable_command::ExecutableCommand;
use crate::layout::{inflate_layout, initialize_cursive_ctx};

mod tasks;
mod executable_command;
mod layout;
mod cursive_formatter;

fn main() {
    let config = tasks::load_task_config().unwrap();

    // Debug layout
    println!("{}", config.layout);

    let mut cmds: Vec<ExecutableCommand> = config.tasks.iter().
        map(|t| ExecutableCommand::new(t.id.clone(),
                                       t.command.clone(),
                                       t.path.clone(),
                                       t.period.clone())).
        collect();

    cmds = cmds.iter().map(|c| c.execute()).collect();

    println!("Setting up siv!");

    let mut siv = initialize_cursive_ctx();
    siv.add_layer(inflate_layout(&mut cmds, config.layout));
    siv.run();
}


