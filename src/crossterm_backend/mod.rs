use regex::{Match, Regex};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use crate::TaskId;

pub struct CrossTermUiContext {
    windows: HashMap<TaskId, Window>,
    rx: Receiver<HashMap<TaskId, String>>,
    fps_tracker: FpsTracker,
}

pub struct Window {
    x: usize,
    y: usize,
    width: usize,
    height: usize
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