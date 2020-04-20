extern crate regex;

use std::collections::HashMap;

use cursive::Cursive;
use cursive::theme::{Color, PaletteColor, Style, Theme};
use cursive::traits::{Resizable, View};
use cursive::utils::span::SpannedString;
use cursive::view::SizeConstraint;
use cursive::views::{DummyView, LinearLayout, TextView};
use log::trace;

use crate::cursive_formatter::format;
use crate::tasks::Layout;

const FPS:u32 = 20; // max=30

pub fn initialize_cursive_ctx() -> Cursive {
    // Creates the cursive root - required for every application.
    let mut siv = Cursive::default();
    let theme = terminal_default_theme(&siv);
    siv.set_theme(theme);
    siv.set_fps(FPS);
    siv
}

pub fn inflate_layout(cmd_text: &HashMap<String, String>, layout: &Layout) -> Box<dyn View> {
    trace!("Inflating {}", layout.kind);

    let inflated : Box<dyn View> = match layout.kind.as_ref() {
        "linearlayout" => build_linear_layout(cmd_text, &layout),
        "textview" => build_text_view(cmd_text, &layout),
        _ => { Box::from(TextView::new(String::from(""))) } // empty view if we can't find one
    };

    return inflated;
}

fn terminal_default_theme(siv: &Cursive) -> Theme {
    // We'll return the current theme with a small modification.
    let mut theme = siv.current_theme().clone();

    theme.palette[PaletteColor::Background] = Color::TerminalDefault;
    theme.palette[PaletteColor::Primary] = Color::TerminalDefault;
    theme.palette[PaletteColor::Secondary] = Color::TerminalDefault;
    theme.palette[PaletteColor::Tertiary] = Color::TerminalDefault;
    theme.palette[PaletteColor::View] = Color::TerminalDefault;
    theme.palette[PaletteColor::Shadow] = Color::TerminalDefault;

    theme
}

fn build_text_view(cmds: &HashMap<String, String>, layout: &Layout) -> Box<dyn View> {
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
        &cmds
    );

    trace!("Creating text view for {}", layout.task_id.as_ref().unwrap_or(&String::from("unknown")));
    trace!("CMD output {:?}", cmd_output.clone());

    let tv = TextView::new(cmd_output.clone());

    Box::from(tv.resized(w_const, h_const))
}

fn build_linear_layout(cmds: &HashMap<String, String>, layout: &Layout) -> Box<dyn View> {
    let mut ll: LinearLayout = match layout.orientation.as_ref().unwrap().as_ref() {
        "horizontal" => LinearLayout::horizontal(),
        _ => LinearLayout::vertical()
    };

    for child in layout.children.as_ref().unwrap_or(&Vec::new()) {
        ll.add_child(inflate_layout(cmds, &child));
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

fn text_for_command(id: &str, cmds: &HashMap<String, String>) -> SpannedString<Style> {
    let raw = match cmds.get(id) {
        Some(s) => s,
        None => ""
    };

    format(raw)
}