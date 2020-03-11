extern crate regex;

use std::convert::TryInto;

use cursive::Cursive;
use cursive::traits::{View, Resizable};
use cursive::theme::{Color, PaletteColor, Theme};
use cursive::view::SizeConstraint;
use cursive::views::{DummyView, LinearLayout, TextView};

use crate::executable_command::ExecutableCommand;
use crate::tasks::Layout;

// TODO: Rename this file to something like "cursive_wrapper" or suchlike, since Layout refers
//       to an object defined in our config files in tasks.rs

pub fn initialize_cursive_ctx() -> Cursive {
    // Creates the cursive root - required for every application.
    let mut siv = Cursive::default();
    let theme = custom_theme_from_cursive(&siv);
    siv.set_theme(theme);
    siv
}

pub fn inflate_layout(cmds: &mut Vec<ExecutableCommand>, layout: Layout) -> Box<dyn View> {
    println!("Inflating {}", layout.kind);

    let inflated : Box<dyn View> = match layout.kind.as_ref() {
        "linearlayout" => build_linear_layout(cmds, layout),
        "textview" => build_text_view(cmds, &layout),
        _ => { Box::from(TextView::new(String::from(""))) } // empty view if we can't find one
    };

    return inflated;
}

fn custom_theme_from_cursive(siv: &Cursive) -> Theme {
    // We'll return the current theme with a small modification.
    let mut theme = siv.current_theme().clone();

    theme.palette[PaletteColor::Background] = Color::TerminalDefault;

    theme
}

fn build_text_view(cmds: &mut Vec<ExecutableCommand>, layout: &Layout) -> Box<dyn View> {
    let h_const = match layout.height {
        Some(h) => SizeConstraint::Fixed(h),
        None => SizeConstraint::Free
    };
    let w_const = match layout.width {
        Some(w) => SizeConstraint::Fixed(w),
        None => SizeConstraint::Free
    };

    let cmd_output = text_for_command(
        layout.task_id.as_ref().unwrap_or(&String::from("")),
        &cmds,
        &h_const,
        &w_const
    );

    println!("Creating text view for {}", layout.task_id.as_ref().unwrap_or(&String::from("unknown")));
    println!("CMD output {}", cmd_output.clone());

    let tv = TextView::new(cmd_output.clone());

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

fn text_for_command(id: &str, cmds: &Vec<ExecutableCommand>, height: &SizeConstraint, width: &SizeConstraint) -> String {
    let raw = match cmds.iter().find(|c| c.id == String::from(id)) {
        Some(cmd) => cmd.output().unwrap_or(String::from("")),
        None => String::from("")
    };

    let max_lines = match height {
        SizeConstraint::Fixed(h) => h.clone(),
        _ => 1000
    };

    let max_width: isize = match width {
        SizeConstraint::Fixed(w) => w.clone().try_into().unwrap_or(1000),
        _ => 1000
    };

    let visible_lines: Vec<String> = raw.split_terminator("\n").map(|line|
                                                                        String::from(line) // vt100_string::slice(line, 0, 1000)
    ).take(max_lines).collect();

    visible_lines.join("\n")
}