use regex::{Regex, Match};
use cursive::utils::markup::StyledString;
use cursive::theme::{Color, BaseColor, Effect, Style};
use cursive::views::TextView;
use crate::cursive_formatter::TextMode::{HI_COLOR, LO_COLOR};

pub fn format(s: &str) -> StyledString {
    let mut styled = StyledString::new();
    let vt100_esc_codes = find_vt100s(s);

    if vt100_esc_codes.is_empty() {
        styled.append(StyledString::plain(s));
    } else {
        let mut head = 0;
        for esc_code_match in vt100_esc_codes {
            let content = &s[head..esc_code_match.start()];
            let style = style_for_vt100_code(esc_code_match.as_str());

            head = esc_code_match.end();

            styled.append(StyledString::styled(content, style));
        }
    }

    styled
}

pub fn strip_vt100(s: &str) -> String {
    let vt100_esc_sequences = Regex::new(r"((\u001b\[|\u009b)[\u0030-\u003f]*[\u0020-\u002f]*[\u0040-\u007e])+").unwrap();
    String::from(vt100_esc_sequences.replace_all(s, ""))
}

pub fn find_vt100s(s: &str) -> Vec<Match> {
    let vt100_regex = Regex::new(r"((\u001b\[|\u009b)[\u0030-\u003f]*[\u0020-\u002f]*[\u0040-\u007e])+").unwrap();
    vt100_regex.find_iter(s).collect()
}

pub fn style_for_vt100_code(esc_code: &str) -> Style {
    let cmd = esc_code.replace("\u{1B}", "");
    let color_match = Regex::new(r"[\d;]*m").unwrap().find(&cmd);

    match color_match {
        Some(clr_code) => style_for_color_code(clr_code.as_str().to_string().replace("m", "").as_str()),
        None => Style::none()
    }
}

enum TextMode {
    LO_COLOR,
    HI_COLOR
}

pub fn style_for_color_code(clr_code: &str) -> Style {
    let cmds: Vec<String> = clr_code.split(";").map(|c| c.to_string()).collect();
    let mut style = Style::none();

    let mut mode = TextMode::LO_COLOR;

    print!("{:?} -> ", cmds);

    for color_cmd in cmds.iter().map(|c| c.parse::<u32>().unwrap()) {
        let next_style: Style = match color_cmd {
            1 => { style.combine(Effect::Bold); style }
            4 => { style.combine(Effect::Underline); style }
            30..=37 => { mode = LO_COLOR; Style::from(Color::Light(BaseColor::from((color_cmd - 30) as u8))) },
            40..=47 => { mode = LO_COLOR; Style::from(Color::Light(BaseColor::from((color_cmd - 40) as u8))) },
            38 => { mode = HI_COLOR; style }
            90..=97 => { mode = LO_COLOR; Style::from(Color::Dark(BaseColor::from((color_cmd - 90) as u8))) },
            100..=107 => { mode = LO_COLOR; Style::from(Color::Dark(BaseColor::from((color_cmd - 100) as u8))) },
            _ => style
        };

        style = Style::merge(&vec!(style, next_style));
    }

    println!("{:?}", style);
    style
}


#[cfg(test)]
mod tests {
    use super::*;

    const VT100_TEST: &str = "T\u{1B}[33mE\u{1B}[96mS\u{1B}[39mT\u{1B}[39m"; // "TEST" interspersed with color codes for VT100 terminals

    fn simple_string() -> String { String::from("simple") }

    fn complex_string() -> String { String::from(VT100_TEST) }

    fn complex_styled_string() -> StyledString {
        let mut ss = StyledString::plain("T");
        ss.append(StyledString::styled("E", Style::from(Color::Light(BaseColor::Yellow))));
        ss.append(StyledString::styled("S", Style::from(Color::Dark(BaseColor::Cyan))));
        ss.append(StyledString::plain("T"));
        ss
    }

    /***
    * strip_vt100
    ***/
    #[test]
    fn strip_vt100_removes_escape_codes() {
        let strp = strip_vt100(&complex_string());
        assert_eq!(strp, "TEST")
    }

    #[test]
    fn strip_vt100_leaves_basic_strings_alone() {
        let strp = strip_vt100(&simple_string());
        assert_eq!(strp, "simple")
    }

    /***
    * format
    ***/

    #[test]
    fn format_leaves_basic_strings_alone() {
        let fmt = format(&simple_string());
        assert_eq!(fmt, StyledString::plain("simple"))
    }

    #[test]
    fn format_adds_color_references_for_vt100_codes() {
        let fmt = format(&complex_string());
        assert_eq!(fmt, complex_styled_string())
    }
}