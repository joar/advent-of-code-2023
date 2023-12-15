use std::collections::Bound;

use std::ops::{Index, Range, RangeBounds};

use std::slice::SliceIndex;

const LEFT_BOTTOM_CORNER: &str = "└";
const RIGHT_BOTTOM_CORNER: &str = "┘";
const HORIZONTAL: &str = "─";
const ARROW_UP: &str = "↑";

pub fn format_text_span<R>(text: &str, range: R) -> String
where
    R: RangeBounds<usize> + SliceIndex<[char], Output = [char]>,
{
    let chars: Vec<char> = text.chars().collect();

    let prefix_range = match range.start_bound() {
        Bound::Included(&x) => ..x,
        Bound::Excluded(&x) => ..x + 1,
        Bound::Unbounded => ..0,
    };
    let suffix_range = match range.end_bound() {
        Bound::Included(x) => x + 1..,
        Bound::Excluded(&x) => x..,
        Bound::Unbounded => chars.len()..,
    };
    let prefix: Vec<char> = chars[prefix_range].to_vec();
    let inner: Vec<char> = chars.index(range).into();
    let suffix: Vec<char> = chars[suffix_range].to_vec();

    String::from_iter(
        prefix.iter().chain(
            ['[']
                .iter()
                .chain(inner.iter().chain([']'].iter().chain(suffix.iter()))),
        ),
    )
}

pub fn format_text_with_marked_span_multiline(text: &str, range: Range<usize>) -> String {
    let span_size = range.end - range.start;
    let marker = match span_size {
        0 => "".to_string(),
        1 => ARROW_UP.to_string(),
        2.. => format!(
            "{}{}{}",
            LEFT_BOTTOM_CORNER,
            HORIZONTAL.repeat(range.end - range.start - 1),
            RIGHT_BOTTOM_CORNER,
        ),
        _ => "?".to_string(),
    };

    format!("{}\n{}{}", text, " ".repeat(range.start), marker)
}

#[cfg(test)]
mod test {
    use crate::utils::format_text_span;
    use aoc2023lib::init_logging;
    use ctor::ctor;
    use paste::paste;

    #[ctor]
    fn init() {
        init_logging();
    }

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
        end_empty: ("01234", 5..5, "01234[]"),
        // reversed_len5: ("01234", 5..0, "]01234["),
        // // should be clamped to max
        // verify_clamped: ("01234", 0..7, "[01234]"),
        // verify_clamped_reverse: ("01234", 7..0, "]01234["),
    }
}
