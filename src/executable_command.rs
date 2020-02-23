use std::process::{Command, Output};
use std::str;

pub struct ExecutableCommand {
    command: String,
    working_dir: String,
    text: Option<String>,
}

impl ExecutableCommand {
    pub fn new(command: String, working_dir: String) -> ExecutableCommand {
        ExecutableCommand {
            command,
            working_dir,
            text: None,
        }
    }

    pub fn execute(&self) -> ExecutableCommand {
        self.capture_output(self.run_command())
    }

    pub fn output(&self) -> Option<String> {
        return self.text.clone();
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
            text,
            command: self.command.clone(),
            working_dir: self.working_dir.clone(),
        };
    }
}