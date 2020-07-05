use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{stdout, Stdout, Write};
use std::rc::{Rc, Weak};
use std::sync::mpsc::Receiver;
use std::time::{Instant, Duration};

use crossterm::cursor::{Hide, MoveTo, MoveToNextLine};
use crossterm::{QueueableCommand, ExecutableCommand, Result};
use crossterm::style::{self, Colorize, Print, PrintStyledContent};
use crossterm::terminal::{Clear, ClearType};
use crossterm::event::{read, Event, poll, KeyModifiers, KeyCode, KeyEvent};
use log::{info, trace};
use regex::{Match, Regex};

use crate::TaskId;
use crate::tasks::Layout;
use crate::widgets::{Dim, LinearLayout, Orientation, TextView, View};

type WindowMap = HashMap<TaskId, Weak<RefCell<TextView>>>;
type RcView = Rc<RefCell<dyn View>>;

pub struct CrossTermUiContext {
    windows: WindowMap,
    top_view: RcView,
    rx: Receiver<HashMap<TaskId, String>>,
    fps_tracker: FpsTracker,
    console_text: String,
    stdout: Stdout,
}

impl CrossTermUiContext {
    pub fn new(layout: Layout, rx: Receiver<HashMap<TaskId, String>>) -> CrossTermUiContext {
        let mut windows = WindowMap::new();
        let top_view = construct_layout(&layout, &mut windows);
        let fps_tracker = FpsTracker { updates: 0.0, elapsed: 0 };
        let console_text = String::new();

        CrossTermUiContext {
            windows,
            top_view,
            rx,
            fps_tracker,
            console_text,
            stdout: stdout()
        }
    }

    pub fn run_ui_loop(&mut self) -> (){
        crossterm::terminal::enable_raw_mode().unwrap();
        self.stdout.
            queue(Hide).unwrap().
            queue(crossterm::terminal::EnterAlternateScreen).unwrap().
            queue(Clear(ClearType::All)).unwrap();

        self.stdout.flush().unwrap();

        let mut last_log = Instant::now();
        loop {
            let start = Instant::now();

            if self.wait_for_updates() {
                self.reinflate_ui().unwrap_or({info!("Failed to reinflate ui!")});
                self.draw_ui().unwrap_or({info!("Failed to draw ui!")});
            }

            self.fps_tracker.elapsed += start.elapsed().as_millis();

            if last_log.elapsed().as_secs() > 10 {
                info!("Refreshes per second = {:.2}", self.fps_tracker.updates / ((self.fps_tracker.elapsed as f64) / 1000.0));
                last_log = Instant::now()
            }
        }
    }

    fn wait_for_updates(&mut self) -> bool {
        self.check_for_keypress().unwrap() || self.check_for_task_updates()
    }

    fn check_for_task_updates(&mut self) -> bool {
        match self.rx.try_recv() {
            Ok(cmd_text) => {
                self.update_output(&cmd_text);
                true
            },
            Err(_) => { false }
        }
    }

    fn check_for_keypress(&mut self) -> Result<bool> {
        // `read()` blocks until an `Event` is available, so call `poll` to check first.
        if poll(Duration::from_millis(100))? {
            let event = read()?;
            match event {
                Event::Key(event) => {
                    match event {
                        // CTRL_C
                        KeyEvent{
                            code: KeyCode::Char('c'),
                            modifiers: KeyModifiers::CONTROL
                        } => {
                            // TODO: Move this out of here.
                            // Reset terminal
                            self.stdout.
                                queue(crossterm::cursor::Show)?.
                                queue(crossterm::terminal::LeaveAlternateScreen)?;
                            self.stdout.flush()?;
                            crossterm::terminal::disable_raw_mode()?;
                            // and uh, exit...
                            panic!("USER QUIT!")
                        },
                        // CTRL_U
                        KeyEvent{
                            code: KeyCode::Char('u'),
                            modifiers: KeyModifiers::CONTROL
                        } => {
                            info!("Clear buffer");
                            self.console_text = String::new();
                        },
                        // ENTER
                        KeyEvent{
                            code: KeyCode::Enter,
                            modifiers: KeyModifiers::NONE
                        } => {
                            info!("Running command: {}", self.console_text);
                            // TODO: Find the ExecutableCommand to load and run it with args.
                            self.console_text = String::new();
                            return Ok(true);
                        },
                        // General key press
                        KeyEvent{
                            code: KeyCode::Char(c),
                            modifiers: KeyModifiers::NONE
                        } => {
                            self.console_text += c.to_string().as_str();
                            return Ok(true);
                        },
                        // I don't care about anything else
                        _ => {}
                    }
                    info!("Key Event: {:?}", event);
                },
                _ => { info!("Other Event: {:?}", event) } // Nothing to do here.
            }
        }
        Ok(false)
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
            let spcs_reqd : usize = (w as usize) - (3 + command.len()); // prompt + length of command

            self.stdout.
                queue(MoveTo(4, 15))?.
                queue(PrintStyledContent("> ".green()))?.
                queue(Print(command))?.
                queue(Print(format!("{:width$}", "", width=spcs_reqd)))?;
        };

        Ok(())
    }

    pub fn update_output(&mut self, output: &HashMap<String, String>) {
        for (task_id, content) in output {
            match self.windows.get(task_id.as_str()) {
                None => {}
                Some(text_view) => {
                    self.fps_tracker.updates += 1.0;
                    match text_view.upgrade() {
                        None => {},
                        Some(tv) => tv.borrow_mut().update_content(content.clone())
                    }
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

