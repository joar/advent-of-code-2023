use anyhow::Context;
use std::cmp::{max, min};
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::ops::Range;
use std::path::Path;
use valuable::Valuable;

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

const LEFT_BOTTOM_CORNER: &'static str = "└";
const RIGHT_BOTTOM_CORNER: &'static str = "┘";
const HORIZONTAL: &'static str = "─";
const ARROW_UP: &'static str = "↑";

const COMBINING_UNDERSCORE: &'static str = "̲";

const COMBINING_DOUBLE_MACRON_BELOW: char = '͟';

pub fn format_text_span(text: &str, range: Range<usize>) -> String {
    let mut chars: Vec<char> = text.chars().collect();

    let ((i0, c0), (i1, c1)) = match range.start <= range.end {
        true => ((range.end, ']'), (range.start, '[')),
        false => ((range.start, '['), (range.end, ']')),
    };

    chars.insert(min(i0, chars.len()), c0);
    chars.insert(min(i1, chars.len() + 1), c1);

    String::from_iter(chars)
}

pub fn format_text_with_marked_span_multiline(text: &str, range: Range<usize>) -> String {
    let span_size = range.end - range.start;
    let marker = match span_size {
        0 => "".to_string(),
        1 => ARROW_UP.to_string(),
        1.. => format!(
            "{}{}{}",
            LEFT_BOTTOM_CORNER,
            HORIZONTAL.repeat(range.end - range.start - 1),
            RIGHT_BOTTOM_CORNER,
        ),
        _ => "?".to_string(),
    };

    format!("{}\n{}{}", text, " ".repeat(range.start), marker)
}

mod test {
    use crate::utils::format_text_span;

    use paste::paste;

    macro_rules! test_format_text_span {
        ($($name:ident: $value:expr,)*) => {
            $(
                paste! {
                    #[test]
                    fn [<test_format_text_span_ $name>]() {
                        let (text, range, expected) = $value;
                        assert_eq!(expected, format_text_span(text, range));
                    }
                }
            )*
        }
    }

    test_format_text_span! {
        empty: ("01234", 0..0, "[]01234"),
        len1: ("01234", 0..1, "[0]1234"),
        len2: ("01234", 0..2, "[01]234"),
        len5: ("01234", 0..5, "[01234]"),
        empty_3: ("01234", 3..3, "012[]34"),
        reversed_len5: ("01234", 5..0, "]01234["),
        end_empty: ("01234", 5..5, "01234[]"),
        // should be clamped to max
        verify_clamped: ("01234", 0..7, "[01234]"),
        verify_clamped_reverse: ("01234", 7..0, "]01234["),
    }
}
