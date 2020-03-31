use std::collections::HashMap;
use std::process::{Command, Output};
use std::str;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::{JoinHandle, sleep};

use crate::executable_command::ExecutableCommand;
use crate::tasks::Task;
use std::time::{SystemTime, Duration};
use log::info;

pub struct TaskRunner {
    pub commands: Vec<ExecutableCommand>,
    tx: Sender<HashMap<String, String>>,
}

impl TaskRunner {
    pub fn new(tasks: Vec<Task>, tx: Sender<HashMap<String, String>>) -> TaskRunner {
        let cmds = tasks.iter().
            map(|t| task_to_command(t)).
            collect();

        TaskRunner { commands: cmds, tx: tx }
    }

    pub fn run(&mut self) {
        let handles: Vec<JoinHandle<()>> = self.commands.iter().map(|cmd|
            self.launch_task_thread(&cmd)
        ).collect();

        for h in handles { h.join().unwrap_or_else(|_| {}); }
    }

    fn launch_task_thread(&self, command: &ExecutableCommand) -> JoinHandle<()> {
        let trx = self.tx.clone();
        let cmd = command.clone();
        info!("spawn {} thread", cmd.id);

        thread::Builder::new().name(cmd.id.clone()).spawn(move ||
            {
                loop {
                    let last_run = SystemTime::now();

                    let mut h = HashMap::new();
                    h.insert(cmd.id.clone(), convert_output(exec_command(cmd.command.clone(), cmd.working_dir.clone())));
                    trx.send(h).unwrap();

                    let nap_millis = cmd.millis_until_next_run(last_run.elapsed().unwrap().as_millis() as u64);
                    let naptime = Duration::from_millis(nap_millis);
                    info!("{} ran for {:?}", cmd.id, last_run.elapsed().unwrap());
                    info!("{} sleeping for {}ms", cmd.id, nap_millis);
                    sleep(naptime);
                }
            }).unwrap()
    }
}

fn convert_output(output: Output) -> String {
    let std_text = match str::from_utf8(&output.stdout) {
        Ok(t) => t.to_owned(),
        Err(_) => String::from("")
    };

    let err_text = match str::from_utf8(&output.stderr) {
        Ok(t) => t.to_owned(),
        Err(_) => String::from("")
    };

    // If we got an error, show that.
    match err_text.len() {
        0 => std_text,
        _ => err_text
    }
}

fn exec_command(command: String, working_dir: String) -> Output {
    let mut splits = command.split(" ").peekable();
    let cmd: String = splits.next().unwrap().to_string();
    let mut args : Vec<String> = Vec::new();
    while splits.peek() != None {
        args.push(splits.next().unwrap().to_string());
    }

    Command::new(vec!(working_dir.clone(), cmd).join("/"))
        .current_dir(working_dir.clone())
        .args(args)
        .output()
        .expect("failed to execute process")
}

fn task_to_command(t: &Task) -> ExecutableCommand {
    ExecutableCommand::new(t.id.clone(),
                           t.command.clone(),
                           t.path.clone(),
                           t.period.clone())
}
