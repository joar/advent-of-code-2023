use anyhow::{Context, Result};

use crate::data::{Bid, Cardish, Hand};

pub fn parse_input<T>(input: &str) -> Result<Vec<(Hand<T>, Bid)>>
where
    T: Cardish,
{
    input.lines().map(parse_line).collect::<Result<Vec<_>>>()
}

pub fn parse_line<T>(line: &str) -> Result<(Hand<T>, Bid)>
where
    T: Cardish,
{
    let (hand_str, bid_str) = line
        .split_once(' ')
        .with_context(|| format!("Could not split {:?} once", line))?;
    Ok((
        Hand::parse(hand_str)?,
        Bid::new(
            bid_str
                .trim()
                .parse::<u32>()
                .with_context(|| format!("Could not parse {:?}", bid_str.trim()))?,
        ),
    ))
}

#[cfg(test)]
mod test {
    use crate::data::{Bid, Card, Type};
    use crate::parse::parse_input;
    use crate::TEST_INPUT;

    #[test]
    fn test_parse_input() {
        let actual: Vec<_> = parse_input::<Card>(TEST_INPUT)
            .unwrap()
            .into_iter()
            .map(|(hand, bid)| (hand.r#type(), bid))
            .collect();
        assert_eq!(
            actual,
            vec![
                (Type::OnePair, Bid::new(765)),
                (Type::ThreeOfAKind, Bid::new(684)),
                (Type::TwoPair, Bid::new(28)),
                (Type::TwoPair, Bid::new(220)),
                (Type::ThreeOfAKind, Bid::new(483)),
            ]
        )
    }
}
