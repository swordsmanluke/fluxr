extern crate regex;

use std::collections::HashMap;

use cursive::{Cursive, CursiveExt};
use cursive::theme::{Color, PaletteColor, Theme};
use cursive::traits::{Resizable, View};
use cursive::view::SizeConstraint;
use cursive::views::{DummyView, LinearLayout, TextView, TextContent, ResizedView};
use log::trace;

use crate::tasks::Layout;

const FPS:u32 = 20; // max=30

pub fn initialize_cursive_ctx() -> Cursive {
    // Creates the cursive root - required for every application.
    let mut siv = Cursive::crossterm().unwrap();
    let theme = terminal_default_theme(&siv);
    siv.set_theme(theme);
    siv.set_fps(FPS);
    siv
}

pub fn inflate_layout(layout: &Layout, windows: &mut HashMap<String, TextContent>) -> Box<dyn View> {
    trace!("Inflating {}", layout.kind);

    let inflated : Box<dyn View> = match layout.kind.as_ref() {
        "linearlayout" => build_linear_layout(&layout, windows),
        "textview" => {
            let (tv, content) = build_text_view(&layout);
            let task_id = layout.task_id.as_ref().unwrap_or(&"uuid".to_string()).to_string();
            windows.insert(task_id, content);
            tv
        },
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

fn build_text_view(layout: &Layout) -> (Box<ResizedView<TextView>>, TextContent) {
    let h_const = match layout.height {
        Some(h) => SizeConstraint::Fixed(h),
        None => SizeConstraint::Free
    };
    let w_const = match layout.width {
        Some(w) => SizeConstraint::Fixed(w),
        None => SizeConstraint::Free
    };

    trace!("Creating text view for {}", layout.task_id.as_ref().unwrap_or(&String::from("unknown")));

    let mut tv = TextView::new("");
    let content = tv.get_shared_content();

    (Box::from(tv.resized(w_const, h_const)), content)
}

fn build_linear_layout(layout: &Layout, windows: &mut HashMap<String, TextContent>) -> Box<dyn View> {
    let mut ll: LinearLayout = match layout.orientation.as_ref().unwrap().as_ref() {
        "horizontal" => LinearLayout::horizontal(),
        _ => LinearLayout::vertical()
    };

    for child in layout.children.as_ref().unwrap_or(&Vec::new()) {
        let child= inflate_layout(&child, windows);
        ll.add_child(child);
        //ll.add_child(DummyView.fixed_width(1).fixed_height(1));
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