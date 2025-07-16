use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "dice.pest"] // path relative to src/
pub struct DiceParser;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Number(i32),
    Dice {
        count: Box<Expr>,
        sides: Box<Expr>,
        modifiers: Vec<DiceModifier>,
    },
    BinaryOp(Box<Expr>, char, Box<Expr>),
}

#[derive(Debug, PartialEq)]
pub enum DiceModifierType {
    KeepHigh,
    KeepLow,
    DropHigh,
    DropLow,
    Explode,
}

#[derive(Debug, PartialEq)]
pub struct DiceModifier {
    pub kind: DiceModifierType,
    pub value: Option<Box<Expr>>,
}

pub fn parse_expressions(pair: pest::iterators::Pair<Rule>) -> Vec<Expr> {
    assert_eq!(pair.as_rule(), Rule::dice_expr);
    pair.into_inner().map(|e| parse_expr(e)).collect()
}

pub fn parse_expr(pair: pest::iterators::Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::number => Expr::Number(pair.as_str().parse::<i32>().unwrap()),

        Rule::dice => {
            let children: Vec<_> = pair.into_inner().collect();

            let mut count = Box::new(Expr::Number(1));
            let mut sides = Box::new(Expr::Number(1));
            let mut modifiers = Vec::new();
            let mut set_sides = false;

            for child in children.into_iter() {
                match child.as_rule() {
                    Rule::dice_modifier => {
                        let mut mod_inner = child.into_inner();
                        let kind_pair = mod_inner.next().unwrap();
                        let kind = match kind_pair.as_rule() {
                            Rule::explode => DiceModifierType::Explode,
                            Rule::keep_high => DiceModifierType::KeepHigh,
                            Rule::keep_low => DiceModifierType::KeepLow,
                            Rule::drop_high => DiceModifierType::DropHigh,
                            Rule::drop_low => DiceModifierType::DropLow,
                            _ => panic!("unknown modifier type!"),
                        };

                        let value = if let Some(v) = mod_inner.next() {
                            Some(Box::new(parse_expr(v)))
                        } else {
                            None
                        };

                        modifiers.push(DiceModifier { kind, value });
                    }
                    Rule::number | Rule::expr => {
                        if set_sides {
                            count = sides;
                            sides = Box::new(parse_expr(child.clone()));
                        } else {
                            sides = Box::new(parse_expr(child.clone()));
                            set_sides = true;
                        }
                    }
                    _ => unreachable!("from dice, {:?}", child.as_rule()),
                }
            }

            Expr::Dice {
                count,
                sides,
                modifiers,
            }
        }

        Rule::expr => {
            let mut inner = pair.into_inner();
            let mut left = parse_expr(inner.next().unwrap());

            while let Some(op) = inner.next() {
                let op_char = op.as_str().chars().next().unwrap();
                let right = parse_expr(inner.next().unwrap());
                left = Expr::BinaryOp(Box::new(left), op_char, Box::new(right));
            }
            left
        }

        _ => unreachable!("from expr, {:?}", pair.as_rule()),
    }
}

pub fn parse(input: &str) -> Result<Vec<Expr>, String> {
    let pairs =
        DiceParser::parse(Rule::dice_expr, input).map_err(|e| format!("Parse error: {}", e))?;

    let pair = pairs.into_iter().next().ok_or("No expressions found")?;

    Ok(parse_expressions(pair))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_and_compare(input: &str, expected: Expr) {
        let pairs = DiceParser::parse(Rule::dice_expr, input)
            .expect("Failed to parse")
            .next()
            .unwrap();

        let exprs = parse_expressions(pairs);

        // Check the first expression only for simplicity
        assert_eq!(exprs[0], expected);
    }

    #[test]
    fn test_simple_number() {
        parse_and_compare("4", Expr::Number(4));
    }

    #[test]
    fn test_simple_dice() {
        parse_and_compare(
            "3d6",
            Expr::Dice {
                count: Box::new(Expr::Number(3)),
                sides: Box::new(Expr::Number(6)),
                modifiers: vec![],
            },
        );
    }

    #[test]
    fn test_dice_with_keep_high() {
        parse_and_compare(
            "4d6kh3",
            Expr::Dice {
                count: Box::new(Expr::Number(4)),
                sides: Box::new(Expr::Number(6)),
                modifiers: vec![DiceModifier {
                    kind: DiceModifierType::KeepHigh,
                    value: Some(Box::new(Expr::Number(3))),
                }],
            },
        );
    }

    #[test]
    fn test_dice_with_explode() {
        parse_and_compare(
            "3d3!",
            Expr::Dice {
                count: Box::new(Expr::Number(3)),
                sides: Box::new(Expr::Number(3)),
                modifiers: vec![DiceModifier {
                    kind: DiceModifierType::Explode,
                    value: None,
                }],
            },
        );
    }

    #[test]
    fn test_multiple_expressions() {
        let input = "3d6 4d3 + 4";

        let pairs = DiceParser::parse(Rule::dice_expr, input)
            .expect("Failed to parse")
            .next()
            .unwrap();

        let exprs = parse_expressions(pairs);
        assert_eq!(exprs.len(), 2);

        assert!(matches!(exprs[0], Expr::Dice { .. }));
        assert!(matches!(exprs[1], Expr::BinaryOp(_, '+', _)));
    }
}
