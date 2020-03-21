use crate::executable_command::ExecutableCommand;
use crate::tasks::Task;
use std::collections::HashMap;

pub struct TaskRunner {
    pub commands: Vec<ExecutableCommand>,
}

impl TaskRunner {
    pub fn new(tasks: Vec<Task>) -> TaskRunner {
        let cmds = tasks.iter().
            map(|t| task_to_command(t)).
            collect();

        TaskRunner{ commands: cmds }
    }

    pub fn update(&self) -> HashMap<String, String> {
        let mut h = HashMap::new();
        self.commands.iter().
            filter(|t| t.ready_to_schedule()).
            for_each(|c| {
                let (id, output) = c.execute();
                h.insert(id, output);
            });

        h
    }
}

fn task_to_command(t: &Task) -> ExecutableCommand {
    ExecutableCommand::new(t.id.clone(),
                           t.command.clone(),
                           t.path.clone(),
                           t.period.clone())
}