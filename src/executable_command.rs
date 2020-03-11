use std::process::{Command, Output};
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};

use regex::Regex;

pub struct ExecutableCommand {
    pub id: String,
    command: String,
    working_dir: String,
    text: Option<String>,
    period: String,
    last_run: SystemTime,
    time_between_runs: u64
}

impl ExecutableCommand {
    pub fn new(id: String, command: String, working_dir: String, period: String) -> ExecutableCommand {
        ExecutableCommand {
            id,
            command,
            working_dir,
            text: None,
            period: period.clone(),
            last_run: UNIX_EPOCH,
            time_between_runs: calc_time_between_runs(period.as_str()),
        }
    }

    pub fn execute(&self) -> ExecutableCommand { self.capture_output(self.run_command()) }

    pub fn output(&self) -> Option<String> {
        return self.text.clone();
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

    fn run_command(&self) -> Output {
        let mut splits = self.command.split(" ").peekable();
        let cmd: String = splits.next().unwrap().to_string();
        let mut args : Vec<String> = Vec::new();
        while splits.peek() != None {
            args.push(splits.next().unwrap().to_string());
        }

        Command::new(vec!(self.working_dir.clone(), cmd).join("/"))
            .current_dir(self.working_dir.clone())
            .args(args)
            .output()
            .expect("failed to execute process")
    }

    fn capture_output(&self, output: Output) -> ExecutableCommand {
        let std_text = str::from_utf8(&output.stdout);
        let err_text = match str::from_utf8(&output.stderr) {
            Ok(t) => Some(t.to_owned()),
            Err(_) => None
        };

        let text = match std_text {
            Ok(t) => Some(t.to_owned()),
            Err(_) => err_text
        };

        return ExecutableCommand {
            id: self.id.clone(),
            text,
            command: self.command.clone(),
            working_dir: self.working_dir.clone(),
            period: self.period.clone(),
            last_run: SystemTime::now(),
            time_between_runs: self.time_between_runs,
        };
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