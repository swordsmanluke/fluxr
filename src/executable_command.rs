use std::str;
use std::time::{SystemTime, UNIX_EPOCH};

use regex::Regex;

pub struct ExecutableCommand {
    pub id: String,
    pub command: String,
    pub working_dir: String,
    pub period: String,
    pub last_run: SystemTime,
    pub time_between_runs: u64
}

impl ExecutableCommand {
    pub fn new(id: String, command: String, working_dir: String, period: String) -> ExecutableCommand {
        ExecutableCommand {
            id,
            command,
            working_dir,
            period: period.clone(),
            last_run: UNIX_EPOCH,
            time_between_runs: calc_time_between_runs(period.as_str()),
        }
    }

    pub fn ready_to_schedule(&self) -> bool {
        self.time_since_last_run() > self.time_between_runs
    }

    fn time_since_last_run(&self) -> u64 {
        match self.last_run.elapsed() {
            Ok(dur) => dur.as_secs(),
            Err(_) => 10_000 // Let's just say it's been awhile
        }
    }
}

fn calc_time_between_runs(period: &str) -> u64{
    let matcher = Regex::new(r"(\d+)([smh]?)").unwrap();

    for c in matcher.captures_iter(period) {
        // Should only be one, but whatever.
        let time = &c[1].parse::<u64>().unwrap();
        let unit = &c[2];

        let mult = match unit {
            "h" => 3600,
            "m" => 60,
            _ => 1 // default to seconds
        };

        return time * mult;
    }

    panic!("Couldn't calculate the time between runs from '{}'", period);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_between_runs_works_for_seconds() {
        assert_eq!(calc_time_between_runs("1s"), 1);
    }

    #[test]
    fn time_between_runs_assumes_seconds() {
        assert_eq!(calc_time_between_runs("12"), 12);
    }

    #[test]
    fn time_between_runs_works_for_minutes() {
        assert_eq!(calc_time_between_runs("1m"), 60);
    }

    #[test]
    fn time_between_runs_works_for_hours() {
        assert_eq!(calc_time_between_runs("1h"), 3600);
    }

    #[test]
    #[should_panic]
    fn time_between_panics_for_bad_pattern() {
        calc_time_between_runs("m");
    }
}