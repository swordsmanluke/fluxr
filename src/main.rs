extern crate regex;
extern crate simplelog;

use std::collections::HashMap;
use std::fs::File;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

use cursive::Cursive;
use simplelog::*;

use crate::terminal_control::{inflate_layout, initialize_cursive_ctx};
use crate::runner::TaskRunner;
use crate::tasks::Layout;
use std::thread::JoinHandle;
use cursive::views::TextContent;
use crate::cursive_formatter::format;
use std::time::Instant;
use log::info;


mod tasks;
mod executable_command;
mod terminal_control;
mod cursive_formatter;
mod runner;

fn main() {
    init_logging();

    let config = tasks::load_task_config().unwrap();
    let layout = config.layout;

    let (tx, rx) = mpsc::channel();

    let mut runner = TaskRunner::new(config.tasks, tx);

    thread::spawn( move || {
        runner.run();
    });

    let uithread = launch_siv(layout, rx);

    uithread.join().unwrap_or({});
}

fn launch_siv(layout: Layout, rx: Receiver<HashMap<String, String>>) -> JoinHandle<()> {
    thread::spawn(move || {
        println!("Setting up siv!");
        let mut siv = initialize_cursive_ctx();
        let mut ctx = UIContenxt::new(layout, rx);
        ctx.create_ui(&mut siv);
        thread::spawn(move || {ctx.run_ui_loop() });
        siv.run();
    })
}

struct UIContenxt {
    layout: Layout,
    windows: HashMap<String, TextContent>,
    rx: Receiver<HashMap<String, String>>,
    updates: f64,
    elapsed: u128
}

impl UIContenxt {
    pub fn new(layout: Layout, rx: Receiver<HashMap<String, String>>) -> UIContenxt {
        UIContenxt {
            layout: layout,
            windows: HashMap::new(),
            rx: rx,
            updates: 0.0,
            elapsed: 0
        }
    }

    pub fn run_ui_loop(&mut self) -> (){
        let mut last_log = Instant::now();
        loop {
            self.updates += 1.0;
            let start = Instant::now();
            self.wait_for_updates();
            self.elapsed += start.elapsed().as_millis();

            if last_log.elapsed().as_secs() > 10 {
                info!("Refreshes per second = {:.2}", self.updates / ((self.elapsed as f64) / 1000.0));
                last_log = Instant::now()
            }
        }
    }

    fn wait_for_updates(&mut self) {
        match self.rx.recv() {
            Ok(cmd_text) => {
                self.update_output(&cmd_text);
            },
            Err(_) => {}
        }
    }

    pub fn update_output(&mut self, output: &HashMap<String, String>) {
        for (task_id, content) in output {
            match self.windows.get(task_id.as_str()) {
                None => {}
                Some(text_content) => {
                    text_content.set_content(format(content.as_str()))
                }
            }
        }
    }

    fn create_ui(&mut self, siv: &mut Cursive) -> () {
        siv.pop_layer();
        let layout = inflate_layout( &self.layout, &mut self.windows);
        siv.add_layer(layout);
    }
}

fn init_logging() {
    CombinedLogger::init(
        vec![
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("log/flux.log").unwrap()),
        ]
    ).unwrap();
}


