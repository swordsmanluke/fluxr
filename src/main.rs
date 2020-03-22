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

use crate::layout::{inflate_layout, initialize_cursive_ctx};
use crate::runner::TaskRunner;
use crate::tasks::Layout;

mod tasks;
mod executable_command;
mod layout;
mod cursive_formatter;
mod runner;

fn main() {
    init_logging();

    let config = tasks::load_task_config().unwrap();
    let layout = config.layout;

    let (tx, rx) = mpsc::channel();

    let runner = TaskRunner::new(config.tasks, tx);

    thread::spawn( move || {
        runner.run_update_loop();
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
    task_output: HashMap<String, String>
}

impl UIContenxt {
    pub fn new(layout: Layout, rx: Receiver<HashMap<String, String>>) -> UIContenxt {
        UIContenxt {
            layout: layout,
            rx: rx,
            task_output: HashMap::new()
        }
    }

    pub fn check_for_ui_update(&mut self, siv: &mut Cursive) -> () {
        match self.rx.try_recv() {
            Ok(cmd_text) => {
                self.update_output(cmd_text);
                self.update_ui(siv);
            }
            Err(_) => {}
        }
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
            WriteLogger::new(LevelFilter::Trace, Config::default(), File::create("log/flux.log").unwrap()),
        ]
    ).unwrap();
}


