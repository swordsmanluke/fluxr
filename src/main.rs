extern crate regex;

use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{DummyView, LinearLayout, TextView, ResizedView};

use executable_command::ExecutableCommand;
use crate::tasks::Layout;
use std::process::exit;
use std::ops::Deref;
use cursive::view::SizeConstraint;

mod vt100_string;

mod tasks;
mod executable_command;

fn main() {
    let config = tasks::load_task_config().unwrap();

    // Debug layout
    // println!("{}", config.layout);

    let mut cmds: Vec<ExecutableCommand> = config.tasks.iter().
        map(|t| ExecutableCommand::new(t.id.clone(),
                                       t.command.clone(),
                                       t.path.clone(),
                                       t.period.clone())).
        collect();

    cmds = cmds.iter().map(|c| c.execute()).collect();

    // Creates the cursive root - required for every application.
    let mut siv = Cursive::default();
    siv.add_layer(inflate_layout(&mut cmds, config.layout));
    siv.run();
}

fn inflate_layout(cmds: &mut Vec<ExecutableCommand>, layout: Layout) -> Box<dyn View> {
    let mut inflated : Box<dyn View> = match layout.kind.as_ref() {
        "linearlayout" => build_linear_layout(cmds, layout),
        "textview" => build_text_view(cmds, &layout),
        _ => { Box::from(TextView::new(String::from(""))) } // empty view if we can't find one
    };

    return inflated;
}

fn build_text_view(cmds: &mut Vec<ExecutableCommand>, layout: &Layout) -> Box<dyn View> {
    let cmd_output = text_for_command(layout.task_id.as_ref().unwrap_or(&String::from("")).as_ref(), &cmds);
    let mut tv = TextView::new(cmd_output);

    let h_const = match layout.height {
        Some(h) => SizeConstraint::Fixed(h),
        None => SizeConstraint::Free
    };
    let w_const = match layout.width {
        Some(w) => SizeConstraint::Fixed(w),
        None => SizeConstraint::Free
    };

    Box::from(tv.resized(w_const, h_const))
}

fn build_linear_layout(cmds: &mut Vec<ExecutableCommand>, layout: Layout) -> Box<dyn View> {
    let mut ll: LinearLayout = match layout.orientation.unwrap().as_ref() {
        "horizontal" => LinearLayout::horizontal(),
        _ => LinearLayout::vertical()
    };

    for child in layout.children.unwrap_or(Vec::new()) {
        ll.add_child(inflate_layout(cmds, child));
        ll.add_child(DummyView.fixed_width(1).fixed_height(1));
    }

    let h_const = match layout.height {
        Some(h) => SizeConstraint::Fixed(h),
        None => SizeConstraint::Free
    };
    let w_const = match layout.width {
        Some(w) => SizeConstraint::Fixed(w),
        None => SizeConstraint::Free
    };

    Box::from(ll.resized(w_const, h_const))
}

fn text_for_command(id: &str, cmds: &Vec<ExecutableCommand>) -> String {
    match cmds.iter().find(|c| c.id == String::from(id)) {
        Some(cmd) => cmd.output().unwrap(),
        None => String::from("")
    }
}
