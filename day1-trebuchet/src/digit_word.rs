use std::fmt::{Display, Formatter};

use valuable::Valuable;

#[derive(Debug, Copy, Clone, Valuable)]
pub enum DigitWord {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

impl DigitWord {
    pub(crate) const fn all() -> [DigitWord; 10] {
        [
            DigitWord::Zero,
            DigitWord::One,
            DigitWord::Two,
            DigitWord::Three,
            DigitWord::Four,
            DigitWord::Five,
            DigitWord::Six,
            DigitWord::Seven,
            DigitWord::Eight,
            DigitWord::Nine,
        ]
    }

    pub(crate) fn int_value(&self) -> u8 {
        match self {
            DigitWord::Zero => 0,
            DigitWord::One => 1,
            DigitWord::Two => 2,
            DigitWord::Three => 3,
            DigitWord::Four => 4,
            DigitWord::Five => 5,
            DigitWord::Six => 6,
            DigitWord::Seven => 7,
            DigitWord::Eight => 8,
            DigitWord::Nine => 9,
        }
    }
    pub(crate) fn str_value(&self) -> &str {
        match self {
            DigitWord::Zero => "zero",
            DigitWord::One => "one",
            DigitWord::Two => "two",
            DigitWord::Three => "three",
            DigitWord::Four => "four",
            DigitWord::Five => "five",
            DigitWord::Six => "six",
            DigitWord::Seven => "seven",
            DigitWord::Eight => "eight",
            DigitWord::Nine => "nine",
        }
    }

    pub(crate) fn char_vec(&self) -> Vec<char> {
        return self.str_value().chars().collect();
    }
}

impl Display for DigitWord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.str_value())
    }
}
