use crate::tasks::Layout;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use crate::terminal_control::inflate_layout;
use cursive::Cursive;
use cursive::views::TextContent;
use crate::cursive_formatter::format;
use std::time::Instant;
use log::info;

pub struct UIContext {
    layout: Layout,
    windows: HashMap<String, TextContent>,
    rx: Receiver<HashMap<String, String>>,
    siv: Cursive,
    updates: f64,
    elapsed: u128
}

impl UIContext {
    pub fn new(layout: Layout, rx: Receiver<HashMap<String, String>>, siv: Cursive) -> UIContext {
        UIContext {
            layout: layout,
            windows: HashMap::new(),
            rx: rx,
            siv: siv,
            updates: 0.0,
            elapsed: 0
        }
    }

    pub fn run_ui_loop(&mut self) -> (){
        let mut last_log = Instant::now();
        self.create_ui();
        loop {
            self.siv.step();
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
                self.siv.refresh();
            },
            Err(_) => {}
        }
    }

    pub fn update_output(&mut self, output: &HashMap<String, String>) {
        for (task_id, content) in output {
            match self.windows.get(task_id.as_str()) {
                None => {}
                Some(text_content) => {
                    self.updates += 1.0;
                    text_content.set_content(format(content.as_str()))
                }
            }
        }
    }

    fn create_ui(&mut self) -> () {
        self.siv.pop_layer();
        let layout = inflate_layout( &self.layout, &mut self.windows);
        self.siv.add_layer(layout);
    }
}
