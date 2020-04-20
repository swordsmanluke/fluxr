extern crate regex;
extern crate simplelog;

use std::collections::HashMap;
use std::fs::File;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

use cursive::Cursive;
use cursive::event::Event;
use simplelog::*;

use crate::terminal_control::{inflate_layout, initialize_cursive_ctx};
use crate::runner::TaskRunner;
use crate::tasks::Layout;
use std::time::{SystemTime, Duration};
use std::alloc::System;

mod tasks;
mod executable_command;
mod terminal_control;
mod cursive_formatter;
mod runner;

const HALF_A_SECOND: Duration = Duration::new(0, 500_000_00);
const TEN_SECONDS: Duration = Duration::new(10, 0);

fn main() {
    init_logging();

    let config = tasks::load_task_config().unwrap();
    let layout = config.layout;

    let (tx, rx) = mpsc::channel();

    let mut runner = TaskRunner::new(config.tasks, tx);

    thread::spawn( move || {
        runner.run();
    });

    let uithread = thread::spawn(move || {
        println!("Setting up siv!");
        let mut siv = initialize_cursive_ctx();
        let mut ctx = UIContenxt::new(layout, rx);
        siv.add_global_callback(Event::Refresh, move |s| { ctx.check_for_ui_update(s) });

        siv.run();
    });

    uithread.join().unwrap_or({});
}

struct UIContenxt {
    layout: Layout,
    rx: Receiver<HashMap<String, String>>,
    task_output: HashMap<String, String>,
    last_refresh: SystemTime,
}

impl UIContenxt {
    pub fn new(layout: Layout, rx: Receiver<HashMap<String, String>>) -> UIContenxt {
        UIContenxt {
            layout: layout,
            rx: rx,
            task_output: HashMap::new(),
            last_refresh: SystemTime::UNIX_EPOCH,
        }
    }

    pub fn check_for_ui_update(&mut self, siv: &mut Cursive) -> () {
        let time_since_last_refresh = self.last_refresh.elapsed().unwrap_or(TEN_SECONDS);
        if time_since_last_refresh > HALF_A_SECOND {
            let updates = self.check_for_task_updates();
            if !updates.is_empty() {
                self.update_output(updates);
                self.update_ui(siv);
            }
        }
    }

    fn check_for_task_updates(&mut self) -> HashMap<String, String> {
        let mut updates = HashMap::new();
        loop {
            match self.rx.try_recv() {
                Ok(cmd_text) => updates.extend(cmd_text),
                Err(_) => break
            }
        }
        updates
    }

    pub fn update_output(&mut self, output: HashMap<String, String>) {
        self.task_output.extend(output);
    }

    fn update_ui(&mut self, siv: &mut Cursive) -> () {
        siv.pop_layer();
        siv.add_layer(inflate_layout(&self.task_output, &self.layout));
    }
}

fn init_logging() {
    CombinedLogger::init(
        vec![
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("log/flux.log").unwrap()),
        ]
    ).unwrap();
}


