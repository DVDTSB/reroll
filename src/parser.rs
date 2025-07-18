use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "dice.pest"]
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
    Repetition {
        count: Box<Expr>,
        expr: Box<Expr>,
        modifiers: Vec<DiceModifier>,
    },
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

pub fn parse_dice_modifier(pair: pest::iterators::Pair<Rule>) -> DiceModifier {
    let mut mod_inner = pair.into_inner();
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
    DiceModifier { kind, value }
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
                        modifiers.push(parse_dice_modifier(child.clone()));
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

        Rule::repetition => {
            let mut children = pair.into_inner();

            let count: Box<Expr> = Box::new(parse_expr(children.next().unwrap().clone()));
            let expr: Box<Expr> = Box::new(parse_expr(children.next().unwrap().clone()));

            let mut modifiers = Vec::new();

            while let Some(child) = children.next() {
                modifiers.push(parse_dice_modifier(child.clone()));
            }

            Expr::Repetition {
                count,
                expr,
                modifiers,
            }
        }
        Rule::add_sub | Rule::mul_div => {
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
    fn test_binary_operation() {
        parse_and_compare(
            "2d6 + 3",
            Expr::BinaryOp(
                Box::new(Expr::Dice {
                    count: Box::new(Expr::Number(2)),
                    sides: Box::new(Expr::Number(6)),
                    modifiers: vec![],
                }),
                '+',
                Box::new(Expr::Number(3)),
            ),
        );
    }

    #[test]
    fn test_nested_binary_ops() {
        parse_and_compare(
            "2d6 + 3 * 2",
            Expr::BinaryOp(
                Box::new(Expr::Dice {
                    count: Box::new(Expr::Number(2)),
                    sides: Box::new(Expr::Number(6)),
                    modifiers: vec![],
                }),
                '+',
                Box::new(Expr::BinaryOp(
                    Box::new(Expr::Number(3)),
                    '*',
                    Box::new(Expr::Number(2)),
                )),
            ),
        );
    }

    #[test]
    fn test_repetition_syntax() {
        parse_and_compare(
            "3(1d6)",
            Expr::Repetition {
                count: Box::new(Expr::Number(3)),
                expr: Box::new(Expr::Dice {
                    count: Box::new(Expr::Number(1)),
                    sides: Box::new(Expr::Number(6)),
                    modifiers: vec![],
                }),
                modifiers: vec![],
            },
        );
    }

    #[test]
    fn test_repetition_with_modifiers() {
        parse_and_compare(
            "2(4d6)kh3",
            Expr::Repetition {
                count: Box::new(Expr::Number(2)),
                expr: Box::new(Expr::Dice {
                    count: Box::new(Expr::Number(4)),
                    sides: Box::new(Expr::Number(6)),
                    modifiers: vec![],
                }),
                modifiers: vec![DiceModifier {
                    kind: DiceModifierType::KeepHigh,
                    value: Some(Box::new(Expr::Number(3))),
                }],
            },
        );
    }

    #[test]
    fn test_multiple_expressions() {
        let input = "3d6 4(1d4) + 4";

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
