extern crate regex;

use std::fs::File;
use std::io::Read;

use serde::{Deserialize};

use executable_command::ExecutableCommand;
use serde::de::Error;

pub mod vt100_string;
mod executable_command;

#[derive(Deserialize)]
struct Config {
    tasks: Vec<Task>,
    windows: Vec<Window>,
}

#[derive(Deserialize)]
struct Task {
    id: String,
    name: String,
    description: String,
    path: String,
    command: String,
    period: String,
}

#[derive(Deserialize)]
struct Window {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    task_id: String,
}

fn main() {
    let config = load_task_config().unwrap();

    let mut cmds: Vec<ExecutableCommand> = config.tasks.iter().
        map(|t| ExecutableCommand::new(t.command.clone(), t.path.clone())).
        collect();

    println!("found {} cmds", cmds.len());

    for _ in 0..10 {
        cmds = cmds.iter().map(|c| c.execute()).collect();

        for cmd in &cmds {
            println!("{}", cmd.output().unwrap_or("No output".to_owned()));
        }
    }

}

fn load_task_config() -> Option<Config> {
    let mut tasks_file = File::open("config/tasks.toml").unwrap();
    let mut toml_tasks = String::new();
    tasks_file.read_to_string(&mut toml_tasks).unwrap();
    let config = toml::from_str(&toml_tasks);
    match config {
        Ok(conf) => Some(conf),
        Err(err) => {
            println!("conf err: {}", err);
            None
        }
    }
}
