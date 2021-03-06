mod input;

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{stdout, Stdout, Write};
use std::rc::{Rc, Weak};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Instant};

use crossterm::cursor::{Hide, MoveTo, MoveToNextLine};
use crossterm::{QueueableCommand, Result};
use crossterm::style::{Colorize, Print, PrintStyledContent};
use crossterm::terminal::{Clear, ClearType};
use log::{info, trace};
use regex::{Match, Regex};

use crate::TaskId;
use crate::tasks::Layout;
use crate::widgets::{Dim, LinearLayout, Orientation, TextView, View};
use std::thread;
use crate::crossterm_backend::input::wait_for_keypress;

type WindowMap = HashMap<TaskId, Weak<RefCell<TextView>>>;
type RcView = Rc<RefCell<dyn View>>;

pub struct CrossTermUiContext {
    windows: WindowMap,
    top_view: RcView,
    command_receiver: Receiver<HashMap<TaskId, String>>,
    command_sender: Sender<HashMap<String, String>>,
    task_sender: Sender<String>,
    fps_tracker: FpsTracker,
    console_text: String,
    stdout: Stdout,
    running: bool,
}

impl CrossTermUiContext {
    pub fn new(layout: Layout, command_receiver: Receiver<HashMap<TaskId, String>>, command_sender: Sender<HashMap<String, String>>, task_sender: Sender<String>) -> CrossTermUiContext {
        let mut windows = WindowMap::new();
        let top_view = construct_layout(&layout, &mut windows);
        let fps_tracker = FpsTracker { updates: 0.0, elapsed: 0 };
        let console_text = String::new();

        CrossTermUiContext {
            windows,
            top_view,
            command_receiver,
            command_sender,
            task_sender,
            fps_tracker,
            console_text,
            stdout: stdout(),
            running: true
        }
    }

    pub fn run_ui_loop(&mut self) -> (){
        crossterm::terminal::enable_raw_mode().unwrap();
        self.stdout.
            queue(Hide).unwrap().
            queue(crossterm::terminal::EnterAlternateScreen).unwrap().
            queue(Clear(ClearType::All)).unwrap();

        self.stdout.flush().unwrap();

        let command_sender = self.command_sender.clone();
        thread::spawn( move || { wait_for_keypress(command_sender) });

        let mut last_log = Instant::now();
        while self.running {
            let start = Instant::now();

            if self.wait_for_updates() {
                self.reinflate_ui().unwrap_or({trace!("Failed to reinflate ui!")});
                self.draw_ui().unwrap_or({trace!("Failed to draw ui!")});
            }

            self.fps_tracker.elapsed += start.elapsed().as_millis();

            if last_log.elapsed().as_secs() > 10 {
                info!("Refreshes per second = {:.2}", self.fps_tracker.updates / ((self.fps_tracker.elapsed as f64) / 1000.0));
                last_log = Instant::now()
            }
        }

        // TODO: Move this out of here.
        // Reset terminal
        self.stdout.
            queue(crossterm::cursor::Show).unwrap().
            queue(crossterm::terminal::LeaveAlternateScreen).unwrap();
        self.stdout.flush().unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        return;
    }

    fn wait_for_updates(&mut self) -> bool {
        match self.command_receiver.recv() {
            Ok(cmd_text) => {
                self.handle_commands(&cmd_text);
                true
            },
            Err(_) => { false }
        }
    }

    fn draw_ui(&mut self) -> Result<()>{
        let output = self.top_view.borrow_mut().render_lines();

        self.stdout.
            queue(MoveTo(0, 0))?;

        for line in output {
            self.stdout.
                queue(Print(line))?.
                queue(MoveToNextLine(1))?;
        }

        self.draw_console()?;

        self.stdout.flush()?;

        Ok(())
    }

    fn draw_console(&mut self) -> Result<()>{
        let command = self.console_text.as_str();

        if !command.is_empty() {
            let (w, _) = crossterm::terminal::size()?;
            let spcs_reqd : usize = (w as usize) - command.len(); // prompt + length of command

            self.stdout.
                queue(MoveTo(4, 17))?.
                queue(PrintStyledContent("> ".green()))?.
                queue(Print(command))?.
                queue(Print(format!("{:width$}", "", width=spcs_reqd)))?;
        };

        Ok(())
    }

    pub fn handle_commands(&mut self, commands: &HashMap<String, String>) {
        for (task_id, content) in commands {
            match task_id.as_str() {
                "system" => {
                    match content.as_str() {
                        "\\u001bQ" => self.running = false, // Shutting down
                        _ => {} // No matching command
                    }
                },
                "console" => {
                    match content.as_str() {
                        "\n" => self.execute_console_cmd(),
                        "\\h" => if self.console_text.len() > 0 { self.console_text = self.console_text[0..self.console_text.len() - 1].to_string() },
                        "\\u001bU" => self.console_text = String::new(),
                        _ => self.console_text += content
                    }
                },
                _ => match self.windows.get(task_id) {
                    Some(text_view) => {
                        self.fps_tracker.updates += 1.0;
                        match text_view.upgrade() {
                            None => {},
                            Some(tv) => tv.borrow_mut().update_content(content.clone())
                        }
                    },
                    None => {}
                }
            }
        }
    }

    fn reinflate_ui(&mut self) -> Result<()> {
        let (w, h) = crossterm::terminal::size()?;
        let dims = (w as usize, h as usize); // Max size of the window.
        info!("Terminal size: {}x{}", w, h);
        self.top_view.borrow_mut().inflate(&dims);
        Ok(())
    }

    fn execute_console_cmd(&mut self) {
        info!("Running {}", self.console_text);
        self.task_sender.send(self.console_text.clone()).unwrap();
        self.console_text = String::new();
    }
}

/*
Not actually "Frames" per second, but "Updates" per second gives "UpsTracker" which
seems more confusing than just making "frames" == "screen updates"
 */
struct FpsTracker {
    updates: f64,
    elapsed: u128
}

pub fn find_vt100s(s: &str) -> Vec<Match> {
    let vt100_regex = Regex::new(r"((\u001b\[|\u009b)[\u0030-\u003f]*[\u0020-\u002f]*[\u0040-\u007e])+").unwrap();
    vt100_regex.find_iter(s).collect()
}

/***
Layout Inflation code
TODO: Move this somewhere more appropriate
 */

pub fn construct_layout(layout: &Layout, windows: &mut WindowMap) -> RcView {
    info!("Building {}:{}", layout.kind, layout.task_id.clone().unwrap_or("".to_string()));

    let constructed: RcView = match layout.kind.as_ref() {
        "linearlayout" => build_linear_layout(&layout, windows),
        "textview" => build_text_view(&layout, windows),
        _ => panic!("Unknown layout {}", layout.kind)
    };

    return constructed;
}

fn build_text_view(layout: &Layout, windows: &mut WindowMap) -> Rc<RefCell<dyn View>> {
    let h_const = match layout.height {
        Some(h) => Dim::Fixed(h),
        None => Dim::WrapContent
    };
    let w_const = match layout.width {
        Some(w) => Dim::Fixed(w),
        None => Dim::WrapContent
    };

    let task_id = layout.task_id.clone().unwrap_or(String::from("unknown"));
    trace!("Creating text view for {}", task_id);
    let tv = Rc::new(RefCell::new(TextView::new(w_const, h_const)));
    windows.insert(task_id.clone(), Rc::downgrade(&tv));

    tv
}

fn build_linear_layout(layout: &Layout, windows: &mut WindowMap) -> RcView {
    let orientation = match layout.orientation.as_ref().unwrap().as_ref() {
        "vertical" => Orientation::VERTICAL,
        _ => Orientation::HORIZONTAL
    };

    let h_const = match layout.height {
        Some(h) => Dim::Fixed(h),
        None => Dim::WrapContent
    };

    let w_const = match layout.width {
        Some(w) => Dim::Fixed(w),
        None => Dim::WrapContent
    };

    let mut ll: LinearLayout = LinearLayout::new(orientation, w_const, h_const);

    for child in layout.children.as_ref().unwrap_or(&Vec::new()) {
        let child= construct_layout(&child, windows);
        ll.add_child(child);
    }

    Rc::new(RefCell::new(ll))
}

