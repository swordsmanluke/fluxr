mod widgets;

extern crate regex;
extern crate simplelog;
extern crate crossterm;

use std::collections::HashMap;
use std::fs::File;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

use simplelog::*;
use log::info;

use crate::terminal_control::initialize_cursive_ctx;
use crate::runner::TaskRunner;
use crate::tasks::Layout;
use std::thread::JoinHandle;
use ui_context::UIContext;
use crate::crossterm_backend::CrossTermUiContext;


mod tasks;
mod executable_command;
mod terminal_control;
mod cursive_formatter;
mod runner;
mod ui_context;
mod crossterm_backend;

pub type TaskId = String;

fn main() {
    init_logging();

    let config = tasks::load_task_config().unwrap();
    let layout = config.layout;

    let (tx, rx) = mpsc::channel();

    let mut runner = TaskRunner::new(config.tasks, tx);

    thread::spawn( move || {
        runner.run();
    });

    if true {
        launch_crossterm(layout, rx).join().unwrap_or({});
    } else {
        // Use the Cursive backend
        launch_siv(layout, rx).join().unwrap_or({});
    }
}

fn launch_crossterm(layout: Layout, rx: Receiver<HashMap<String, String>>) -> JoinHandle<()> {
    thread::spawn(move || {
        info!("Setting up crossterm!");
        let mut ctx = CrossTermUiContext::new(layout, rx);
        ctx.run_ui_loop();
    })
}

fn launch_siv(layout: Layout, rx: Receiver<HashMap<String, String>>) -> JoinHandle<()> {
    thread::spawn(move || {
        info!("Setting up siv!");
        let siv = initialize_cursive_ctx();
        let mut ctx = UIContext::new(layout, rx, siv);
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


