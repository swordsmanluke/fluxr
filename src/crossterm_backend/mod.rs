use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{stdout, Stdout, Write};
use std::rc::{Rc, Weak};
use std::sync::mpsc::Receiver;
use std::time::Instant;

use crossterm::cursor::MoveTo;
use crossterm::style::Print;
use log::{info, trace};
use regex::{Match, Regex};

use crate::TaskId;
use crate::tasks::Layout;
use crate::widgets::{Dim, Dimensions, LinearLayout, Orientation, TextView, View};
use crossterm::QueueableCommand;
use crossterm::terminal::{Clear, ClearType};

type WindowMap = HashMap<TaskId, Weak<RefCell<TextView>>>;
type RcView = Rc<RefCell<dyn View>>;

pub struct CrossTermUiContext {
    windows: WindowMap,
    top_view: RcView,
    rx: Receiver<HashMap<TaskId, String>>,
    fps_tracker: FpsTracker,
    stdout: Stdout,
}

impl CrossTermUiContext {
    pub fn new(layout: Layout, rx: Receiver<HashMap<TaskId, String>>) -> CrossTermUiContext {
        let mut windows = WindowMap::new();
        let top_view = construct_layout(&layout, &mut windows);
        let fps_tracker = FpsTracker { updates: 0.0, elapsed: 0 };

        CrossTermUiContext {
            windows,
            top_view,
            rx,
            fps_tracker,
            stdout: stdout()
        }
    }

    pub fn run_ui_loop(&mut self) -> (){
        self.stdout.queue(Clear(ClearType::All)).unwrap();
        self.stdout.flush().unwrap();

        let mut last_log = Instant::now();
        loop {
            let start = Instant::now();
            self.wait_for_updates();
            self.reinflate_ui();
            self.draw_ui();
            self.fps_tracker.elapsed += start.elapsed().as_millis();

            if last_log.elapsed().as_secs() > 10 {
                info!("Refreshes per second = {:.2}", self.fps_tracker.updates / ((self.fps_tracker.elapsed as f64) / 1000.0));
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

    fn draw_ui(&mut self) {
        let output = self.top_view.borrow_mut().render();
        self.stdout.
            queue(MoveTo(0, 0)).unwrap().
            queue(Print(output)).unwrap();

        self.stdout.flush().unwrap();
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

    fn reinflate_ui(&mut self) -> () {
        let (w, h) = terminal_size::terminal_size().unwrap();
        let dims = (w.0 as usize, h.0 as usize); // Max size of the window.
        info!("Terminal size: {}x{}", w.0, h.0);
        self.top_view.borrow_mut().inflate(&dims);
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

