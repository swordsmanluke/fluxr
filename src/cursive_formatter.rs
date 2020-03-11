use regex::Regex;

pub fn format(s: &str) -> String {
    strip_vt100(s)
}

pub fn strip_vt100(s: &str) -> String {
    let vt100_esc_sequences = Regex::new(r"((\u001b\[|\u009b)[\u0030-\u003f]*[\u0020-\u002f]*[\u0040-\u007e])+").unwrap();

    String::from(vt100_esc_sequences.replace_all(s, ""))
}


#[cfg(test)]
mod tests {
    use super::*;

    const VT100_TEST: &str = "T\u{1B}[33mE\u{1B}[96mS\u{1B}[39mT\u{1B}[39m"; // "TEST" interspersed with color codes for VT100 terminals

    fn simple_string() -> String { String::from("simple") }

    fn complex_string() -> String { String::from(VT100_TEST) }

    #[test]
    fn strip_vt100_removes_escape_codes() {
        let slc = strip_vt100(&complex_string());
        assert_eq!(slc, "TEST")
    }

    #[test]
    fn strip_vt100_leaves_basic_strings_alone() {
        let slc = strip_vt100(&simple_string());
        assert_eq!(slc, "simple")
    }
}