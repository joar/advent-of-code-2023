use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub enum CalibrationDigit {
    AsDigit {
        value: u8,
        range: RangeInclusive<usize>,
    },
    AsWord {
        value: u8,
        range: RangeInclusive<usize>,
    },
}
