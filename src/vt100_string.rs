use regex::{Match, Regex};

pub fn slice(s: &str, start: isize, end: isize) -> String {
    let ustart: usize = if start < 0 { start + s.len() as isize } else { start } as usize;
    let uend: usize = if end < 0 { end + s.len() as isize } else { end } as usize;

    if ustart > s.len() { return String::from("") }

    let vt100_regex = vt100_esc_sequences();
    let mut matches = Vec::new();
    vt100_regex.find_iter(s).for_each(|m| matches.push(m));

    return vt100_slice(s, matches, ustart, uend);
}

fn vt100_slice(s: &str, matches: Vec<Match>, start: usize, end: usize) -> String{
    let stripped_string = String::from(vt100_esc_sequences().replace_all(s, ""));
    let offsets = build_regex_match_offsets(&stripped_string, matches.clone());

    let new_start = offsets[start];
    // If `end` asks for the last char of the string, grab any following VT100 codes as well.
    let new_end = if end == stripped_string.len() - 1 { s.len() } else { offsets[end] + 1 };

    let mut sliced_str = String::new();

    matches.iter().
        filter(|m| m.start() < new_start).
        map(|m| m.as_str()).
        for_each(|s| sliced_str.push_str(&s));

    sliced_str.push_str(&s[new_start..new_end]);
    return sliced_str;
}

// For the given string and Regex captures removed from it, calculate a vector
// which contains a list of offsets to add to each character.
// e.g. a string "-1-2-3-4-", which has had "-" stripped, becomes "1234", with a
// set of Captures at indices 0, 3, 5 & 7 of 1 character each.
// This function should then output a Vector like [1, 3, 5, 7], which represents
// the mapping of the non-removed characters to their position in the original
// string.
fn build_regex_match_offsets(s: &String, matches: Vec<Match>) -> Vec<usize>{
    let mut offsets= vec![0; s.len()];
    let mut cur_offset: usize = 0;
    let mut next_offset = cur_offset.clone();
    let mut i = 0;

    for _ in s.chars() {
        matches.iter().
            map(|c| get_match_bounds(c)).
            filter(|v| match_bounds_offset(cur_offset, v[0], v[1])).
            for_each(|v| next_offset = next_offset + v[1]-v[0]);

        offsets[i] = next_offset;
        next_offset += 1;  // Always add 1 for the current char
        cur_offset = next_offset.clone();

        i += 1;
    }

    return offsets;
}

fn match_bounds_offset(cur_offset: usize, start: usize, end: usize) -> bool {
    start <= cur_offset && end > cur_offset
}

fn get_match_bounds(c: &Match) -> Vec<usize> {
    let start = c.start();
    let end = c.end();
    vec!(start, end)
}

/*****
* TESTS
*****/

fn vt100_esc_sequences() -> Regex {
    Regex::new(r"((\u001b\[|\u009b)[\u0030-\u003f]*[\u0020-\u002f]*[\u0040-\u007e])+").unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    const VT100_TEST: &str = "T\u{1B}[33mE\u{1B}[96mS\u{1B}[39mT\u{1B}[39m"; // "TEST" interspersed with color codes for VT100 terminals

    fn simple_string() -> String { String::from("simple") }
    fn complex_string() -> String { String::from(VT100_TEST) }

    #[test]
    fn simple_slice_works() {
        let simple = simple_string();
        let slc = slice(&simple, 1, 2);
        assert_eq!(slc, "im")
    }

    #[test]
    fn simple_slice_works_with_offset() {
        let simple = simple_string();
        let slc = slice(&simple, -2, -1);

        assert_eq!(slc, "le");
    }

    #[test]
    fn vt100_slice_works() {
        let complex = complex_string();
        let slc = slice(&complex, 0, 2);

        assert_eq!(slc, "T\u{1B}[33mE\u{1B}[96mS")
    }

    #[test]
    fn vt100_slice_works_in_middle_of_string() {
        let complex = complex_string();
        let slc = slice(&complex, 1, 2);

        assert_eq!(slc, "\u{1B}[33mE\u{1B}[96mS")
    }

    #[test]
    fn vt100_slice_works_at_end_of_string() {
        let complex = complex_string();
        let slc = slice(&complex, 1, 3);

        assert_eq!(slc, "\u{1B}[33mE\u{1B}[96mS\u{1B}[39mT\u{1B}[39m")
    }

    #[test]
    fn building_capture_offsets_works() {
        let start = "-1-2-3-4-";
        let regex = Regex::new(r"-").unwrap();
        let captures = regex.find_iter(start);
        let stripped = String::from(regex.replace_all(start, ""));

        let mut matches = Vec::new();
        captures.for_each(|m| matches.push(m));

        let actual= build_regex_match_offsets(&stripped, matches);
        assert_eq!(actual, vec!(1, 3, 5, 7))
    }
}
