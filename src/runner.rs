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
use std::collections::binary_heap::BinaryHeap;

pub struct TaskRunner {
    pub commands: BinaryHeap<ExecutableCommand>,
    tx: Sender<HashMap<String, String>>,
}

impl TaskRunner {
    pub fn new(tasks: Vec<Task>, tx: Sender<HashMap<String, String>>) -> TaskRunner {
        let cmds = tasks.iter().
            map(|t| task_to_command(t)).
            collect();

        TaskRunner { commands: cmds, tx: tx }
    }

    pub fn run_update_loop(&mut self) {
        loop {
            while self.head().ready_to_schedule() {
                let cmd = self.commands.pop().unwrap();
                if cmd.ready_to_schedule() { info!("Running {}!", cmd.id) } else { info!("WTF - running {} anyway?", cmd.id) }
                self.commands.push(
                    match  cmd.ready_to_schedule() {
                        true  => { self.launch_task_thread(&cmd); cmd.with_last_run_at(SystemTime::now()) },
                        false => { cmd }
                    });
            };

            info!("Sleeping for {}ms", self.head().millis_until_next_run());
            sleep(Duration::from_millis(self.head().millis_until_next_run() as u64));
            info!("Awake! Top task {} is ready? {} Next run in {}ms", self.head().id, self.head().ready_to_schedule(), self.head().millis_until_next_run());
        }
    }

    fn head(&self) -> &ExecutableCommand {
        &self.commands.peek().unwrap()
    }

    fn launch_task_thread(&self, cmd: &ExecutableCommand) -> JoinHandle<()> {
        let trx = self.tx.clone();
        let id = cmd.id.clone();
        let command = cmd.command.clone();
        let working_dir = cmd.working_dir.clone();

        thread::spawn(move ||
            {
                let mut h = HashMap::new();
                h.insert(id, convert_output(exec_command(command, working_dir)));
                trx.send(h).unwrap();
            })
    }
}

fn convert_output(output: Output) -> String {
    let std_text = str::from_utf8(&output.stdout);
    let err_text = match str::from_utf8(&output.stderr) {
        Ok(t) => t.to_owned(),
        Err(_) => String::from("")
    };

    match std_text {
        Ok(t) => t.to_owned(),
        Err(_) => err_text
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