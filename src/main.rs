mod widgets;

extern crate regex;
extern crate simplelog;
extern crate crossterm;

use std::collections::HashMap;
use std::fs::File;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use simplelog::*;
use log::info;

use crate::runner::TaskRunner;
use crate::tasks::Layout;
use std::thread::JoinHandle;
use crate::crossterm_backend::CrossTermUiContext;


mod tasks;
mod executable_command;
mod runner;
mod crossterm_backend;

pub type TaskId = String;

pub struct Channel<T> {
    pub tx: Sender<T>,
    pub rx: Receiver<T>
}

impl<T> Channel<T> {
    pub fn new(tx: Sender<T>, rx: Receiver<T>) -> Channel<T> {
        Channel { tx, rx }
    }

    pub fn from(tuple: (Sender<T>, Receiver<T>)) -> Channel<T> {
        Channel::new(tuple.0, tuple.1)
    }
}

fn main() {
    init_logging();

    let config = tasks::load_task_config().unwrap();
    let layout = config.layout;

    let system_command_channel = Channel::from(mpsc::channel());
    let task_running_channel = Channel::from(mpsc::channel());

    let mut runner = TaskRunner::new(config.tasks, system_command_channel.tx.clone(), task_running_channel.rx);

    thread::spawn( move || { runner.run(); });

    launch_crossterm(layout,
                     system_command_channel.rx,
                     system_command_channel.tx,
                     task_running_channel.tx.clone()).join().unwrap_or({});
}

fn launch_crossterm(layout: Layout,
                    command_receiver: Receiver<HashMap<String, String>>,
                    command_sender: Sender<HashMap<String, String>>,
                    task_sender: Sender<String>) -> JoinHandle<()> {
    thread::spawn(move || {
        info!("Setting up crossterm!");
        let mut ctx = CrossTermUiContext::new(layout, command_receiver, command_sender, task_sender);
        ctx.run_ui_loop();
    })
}

fn init_logging() {
    CombinedLogger::init(
        vec![
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("log/flux.log").unwrap()),
        ]
    ).unwrap();
}


