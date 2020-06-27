use cursive::theme::{BaseColor, Color, Effect, Style};
use cursive::utils::markup::StyledString;
use regex::{Match, Regex};

pub fn format(s: &str) -> StyledString {
    let mut styled = StyledString::new();
    let vt100_esc_codes = find_vt100s(s);

    if vt100_esc_codes.is_empty() {
        styled.append(StyledString::plain(s));
    } else {
        let mut style_to_apply:Match = vt100_esc_codes.first().unwrap().clone();

        if style_to_apply.start() > 0 {
            let content = &s[0..style_to_apply.start()];
            let to_add = StyledString::plain(content);
            styled.append(to_add);
        }

        let mut head = style_to_apply.end();
        for esc_code_match in vt100_esc_codes {
            if esc_code_match == style_to_apply { continue; } // This will only be true the first time through. I couldn't get .skip to work.

            let content = &s[head..esc_code_match.start()];
            let style = style_for_vt100_code(style_to_apply.as_str());
            let to_add = StyledString::styled(content, style);
            styled.append(to_add);

            head = esc_code_match.end();
            style_to_apply = esc_code_match; // Whatever we just matched will be enforced for the next block of the string
        }

        if head < s.len() {
            let content = &s[head..s.len()];
            let style = style_for_vt100_code(style_to_apply.as_str());
            styled.append(StyledString::styled(content, style));
        }
    }

    styled
}

pub fn find_vt100s(s: &str) -> Vec<Match> {
    let vt100_regex = Regex::new(r"((\u001b\[|\u009b)[\u0030-\u003f]*[\u0020-\u002f]*[\u0040-\u007e])+").unwrap();
    vt100_regex.find_iter(s).collect()
}

pub fn style_for_vt100_code(esc_code: &str) -> Style {
    let cmd = esc_code.replace("\u{1B}", "");
    let color_match = Regex::new(r"[\d;]*m").unwrap().find_iter(&cmd).last();

    match color_match {
        Some(clr_code) => style_for_color_code(clr_code.as_str().to_string().replace("m", "").as_str()),
        None => Style::none()
    }
}

pub fn style_for_color_code(clr_code: &str) -> Style {
    let cmds: Vec<String> = clr_code.split(";").map(|c| c.to_string()).collect();
    let mut style = Style::none();


    for color_cmd in cmds.iter().map(|c| c.parse::<u32>().unwrap()) {
        let next_style: Style = match color_cmd {
            1 => { style.combine(Effect::Bold); style }
            4 => { style.combine(Effect::Underline); style }
            30..=37 => { Style::from(Color::Dark(BaseColor::from((color_cmd - 30) as u8))) },
            40..=47 => { Style::from(Color::Dark(BaseColor::from((color_cmd - 40) as u8))) },
            38 => { style }
            90..=97 => { Style::from(Color::Light(BaseColor::from((color_cmd - 90) as u8))).combine(Effect::Bold) },
            100..=107 => { Style::from(Color::Light(BaseColor::from((color_cmd - 100) as u8))).combine(Effect::Bold) },
            _ => style
        };

        style = Style::merge(&vec!(style, next_style));
    }

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
        ss.append(StyledString::styled("E", Style::from(Color::Dark(BaseColor::Yellow))));
        ss.append(StyledString::styled("S", Style::from(Color::Light(BaseColor::Cyan)).combine(Effect::Bold)));
        ss.append(StyledString::plain("T"));
        ss
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