use std::str;
use std::time::{SystemTime, UNIX_EPOCH};

use regex::Regex;
use std::cmp::Ordering;

#[derive(Eq)]
pub struct ExecutableCommand {
    pub id: String,
    pub command: String,
    pub working_dir: String,
    pub period: String,
    pub last_run: SystemTime,
    pub time_between_runs: u128
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
        self.millis_until_next_run() <= 0
    }

    pub fn millis_until_next_run(&self) -> u128 {
        let last_run = self.last_run.elapsed().unwrap().as_millis();
        match last_run > self.time_between_runs
        {
            true => 0,
            false => self.time_between_runs - last_run
        }

    }

    pub fn with_last_run_at(&self, last_run: SystemTime) -> ExecutableCommand {
        ExecutableCommand {
            id: self.id.clone(),
            command: self.command.clone(),
            working_dir: self.working_dir.clone(),
            period: self.period.clone(),
            last_run: last_run,
            time_between_runs: self.time_between_runs,
        }
    }
}

impl Ord for ExecutableCommand {
    fn cmp(&self, other: &Self) -> Ordering {
        other.millis_until_next_run().cmp(&self.millis_until_next_run())
    }
}

impl PartialOrd for ExecutableCommand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ExecutableCommand {
    fn eq(&self, other: &Self) -> bool {
        self.last_run == other.last_run
    }
}

fn calc_time_between_runs(period: &str) -> u128{
    let matcher = Regex::new(r"(\d+)([smh]?)").unwrap();

    for c in matcher.captures_iter(period) {
        // Should only be one, but whatever.
        let time = &c[1].parse::<u128>().unwrap();
        let unit = &c[2];

        let mult = match unit {
            "h" => 3600000,
            "m" => 60000,
            _ => 1000 // default to milliseconds
        };

        return time * mult;
    }

    panic!("Couldn't calculate the time between runs from '{}'", period);
}


#[cfg(test)]
mod tests {
    use super::*;

    fn create_with_last_run(period: String, last_run: SystemTime) -> ExecutableCommand {
        create_with_period(period).with_last_run_at(last_run)
    }

    fn create_with_period(period: String) -> ExecutableCommand {
        ExecutableCommand::new("task 1".to_string(),
                               "cmd".to_string(),
                               ".".to_string(),
                               period)
    }

    #[test]
    fn time_between_runs_works_for_seconds() {
        assert_eq!(calc_time_between_runs("1s"), 1000);
    }

    #[test]
    fn time_between_runs_assumes_seconds() {
        assert_eq!(calc_time_between_runs("12"), 12000);
    }

    #[test]
    fn time_between_runs_works_for_minutes() {
        assert_eq!(calc_time_between_runs("1m"), 60000);
    }

    #[test]
    fn time_between_runs_works_for_hours() {
        assert_eq!(calc_time_between_runs("1h"), 3600000);
    }

    #[test]
    #[should_panic]
    fn time_between_panics_for_bad_pattern() {
        calc_time_between_runs("m");
    }

    #[test]
    fn next_to_run_should_be_larger() {
        // Larger to ensure it's at the top of the binary maxheap
        let now = SystemTime::now();
        let ready_in_one_sec = create_with_last_run("1s".to_string(), now);
        let ready_in_two_sec = create_with_last_run("2s".to_string(),now);

        assert_eq!( ready_in_one_sec > ready_in_two_sec, true);
    }

    #[test]
    fn should_compare_equally_when_schedule_is_the_same() {
        let ready_in_one_sec = create_with_period("1s".to_string());
        let ready_in_two_sec = create_with_period("2s".to_string());

        assert_eq!( ready_in_one_sec == ready_in_two_sec, true);
    }
}