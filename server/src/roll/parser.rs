use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, char, u16, usize},
    combinator::opt,
};

use crate::roll::{DiceRoll, Roll, TableRoll};

fn parse_mod(input: &str) -> IResult<&str, u16> {
    let (input, _) = char('+').parse(input)?;
    let (input, modifier) = u16(input)?;

    Ok((input, modifier))
}

fn parse_dice_roll(input: &str) -> IResult<&str, Roll> {
    let (input, count) = usize(input)?;
    let (input, _) = char('d').parse(input)?;
    let (input, die) = u16(input)?;
    let (input, modifier) = opt(parse_mod).parse(input)?;

    Ok((
        input,
        Roll::Dice(DiceRoll {
            die,
            count,
            modifier: modifier.unwrap_or(0),
        }),
    ))
}

fn parse_table_roll(input: &str) -> IResult<&str, Roll> {
    let (input, _) = tag("table:").parse(input)?;
    let (input, name) = alpha1(input)?;

    Ok((
        input,
        Roll::Table(TableRoll {
            name: name.to_owned(),
        }),
    ))
}

pub fn parse_roll(input: &str) -> IResult<&str, Roll> {
    alt((parse_dice_roll, parse_table_roll)).parse(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_roll() {
        assert_eq!(
            parse_roll("4d6"),
            Ok((
                "",
                Roll::Dice(DiceRoll {
                    die: 6,
                    count: 4,
                    modifier: 0
                })
            ))
        )
    }

    #[test]
    fn parses_roll_with_modifier() {
        assert_eq!(
            parse_roll("6d20+4"),
            Ok((
                "",
                Roll::Dice(DiceRoll {
                    die: 20,
                    count: 6,
                    modifier: 4
                })
            ))
        )
    }
}
