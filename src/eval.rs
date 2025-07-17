use crate::parser::{DiceModifier, DiceModifierType, Expr};

#[derive(Debug)]
pub enum EvalResult {
    Rolls(Vec<i32>),
    Number(i32),
}

impl EvalResult {
    pub fn to_number(&self) -> i32 {
        match self {
            EvalResult::Number(n) => *n,
            EvalResult::Rolls(v) => v.iter().sum(),
        }
    }
}

pub fn roll(sides: i32) -> i32 {
    fastrand::i32(1..=sides)
}

pub fn eval_expr(expr: &Expr) -> EvalResult {
    match expr {
        Expr::Number(n) => EvalResult::Number(*n),
        Expr::Dice {
            count,
            sides,
            modifiers,
        } => eval_dice(count, sides, modifiers),
        Expr::BinaryOp(exp1, op, exp2) => eval_op(exp1, op, exp2),
        Expr::Repetition {
            count,
            expr,
            modifiers,
        } => eval_rep(count, expr, modifiers),
    }
}

pub fn eval_op(exp1: &Expr, op: &char, exp2: &Expr) -> EvalResult {
    let exp1 = eval_expr(exp1).to_number();
    let exp2 = eval_expr(exp2).to_number();

    let result = match op {
        '+' => exp1 + exp2,
        '-' => exp1 - exp2,
        '*' => exp1 * exp2,
        '/' => {
            if exp2 == 0 {
                panic!("division by zero!");
            }
            exp1 / exp2
        }
        _ => panic!("unsupported operation: {}", op),
    };
    EvalResult::Number(result)
}

pub fn eval_dice(count: &Expr, sides: &Expr, modifiers: &[DiceModifier]) -> EvalResult {
    let count = eval_expr(count).to_number();
    let sides = eval_expr(sides).to_number();

    let mut rolls: Vec<i32> = Vec::new();

    for _ in 0..count {
        rolls.push(roll(sides));
    }

    eval_modifiers(rolls, modifiers, Some(sides))
}

pub fn eval_rep(count: &Expr, expr: &Expr, modifiers: &[DiceModifier]) -> EvalResult {
    let count = eval_expr(count).to_number();
    let mut result: Vec<i32> = Vec::new();

    for _ in 0..count {
        result.push(eval_expr(expr).to_number());
    }

    eval_modifiers(result, modifiers, None)
}

pub fn eval_modifiers(
    mut rolls: Vec<i32>,
    modifiers: &[DiceModifier],
    sides: Option<i32>,
) -> EvalResult {
    for modifier in modifiers.iter() {
        let value = match &modifier.value {
            Some(expr_box) => eval_expr(&**expr_box).to_number(),
            None => {
                if modifier.kind == DiceModifierType::Explode {
                    sides.expect("Explode requires number of sides.")
                } else {
                    panic!(
                        "All dice modifiers (except explode) must be followed by a value. E.g. 4d6kh3"
                    );
                }
            }
        };

        rolls = match modifier.kind {
            DiceModifierType::KeepHigh => keep_high(rolls, value),
            DiceModifierType::KeepLow => keep_low(rolls, value),
            DiceModifierType::DropHigh => drop_high(rolls, value),
            DiceModifierType::DropLow => drop_low(rolls, value),
            DiceModifierType::Explode => explode(rolls, sides.expect("Missing sides"), value),
        };
    }

    EvalResult::Rolls(rolls)
}

fn keep_high(mut rolls: Vec<i32>, count: i32) -> Vec<i32> {
    rolls.sort_by(|a, b| b.cmp(a));
    rolls.truncate(count as usize);
    rolls
}

fn keep_low(mut rolls: Vec<i32>, count: i32) -> Vec<i32> {
    rolls.sort();
    rolls.truncate(count as usize);
    rolls
}

fn drop_high(mut rolls: Vec<i32>, count: i32) -> Vec<i32> {
    rolls.sort_by(|a, b| b.cmp(a));
    rolls.drain(0..count as usize);
    rolls
}

fn drop_low(mut rolls: Vec<i32>, count: i32) -> Vec<i32> {
    rolls.sort();
    rolls.drain(0..count as usize);
    rolls
}

fn explode(mut rolls: Vec<i32>, sides: i32, threshold: i32) -> Vec<i32> {
    let mut i = 0;
    while i < rolls.len() {
        while rolls[i] >= threshold {
            let new_roll = roll(sides);
            rolls.push(new_roll);
            if new_roll < threshold {
                break;
            }
        }
        i += 1;
    }
    rolls
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{DiceModifier, DiceModifierType, Expr};

    fn num(n: i32) -> Expr {
        Expr::Number(n)
    }

    fn dice(count: i32, sides: i32, modifiers: Vec<DiceModifier>) -> Expr {
        Expr::Dice {
            count: Box::new(num(count)),
            sides: Box::new(num(sides)),
            modifiers,
        }
    }

    fn rep(count: i32, expr: Expr, modifiers: Vec<DiceModifier>) -> Expr {
        Expr::Repetition {
            count: Box::new(num(count)),
            expr: Box::new(expr),
            modifiers,
        }
    }

    fn binop(lhs: Expr, op: char, rhs: Expr) -> Expr {
        Expr::BinaryOp(Box::new(lhs), op, Box::new(rhs))
    }

    fn modifier(kind: DiceModifierType, val: Option<i32>) -> DiceModifier {
        DiceModifier {
            kind,
            value: val.map(|v| Box::new(num(v))),
        }
    }

    #[test]
    fn eval_number() {
        let expr = num(42);
        assert_eq!(eval_expr(&expr).to_number(), 42);
    }

    #[test]
    fn eval_addition() {
        let expr = binop(num(2), '+', num(3));
        assert_eq!(eval_expr(&expr).to_number(), 5);
    }

    #[test]
    fn eval_multiplication_precedence() {
        let expr = binop(num(2), '+', binop(num(3), '*', num(4)));
        assert_eq!(eval_expr(&expr).to_number(), 14);
    }

    #[test]
    fn eval_simple_dice_roll() {
        let expr = dice(2, 6, vec![]);
        match eval_expr(&expr) {
            EvalResult::Rolls(rolls) => {
                assert_eq!(rolls.len(), 2);
                assert!(rolls.iter().all(|&r| (1..=6).contains(&r)));
            }
            _ => panic!("Expected dice rolls"),
        }
    }

    #[test]
    fn eval_repetition_roll() {
        let inner_dice = dice(1, 6, vec![]);
        let expr = rep(3, inner_dice, vec![]);
        match eval_expr(&expr) {
            EvalResult::Rolls(rolls) => {
                assert_eq!(rolls.len(), 3);
                assert!(rolls.iter().all(|&r| (1..=6).contains(&r)));
            }
            _ => panic!("Expected repetition result"),
        }
    }

    #[test]
    fn keep_high_modifier_works() {
        let expr = Expr::Dice {
            count: Box::new(num(5)),
            sides: Box::new(num(6)),
            modifiers: vec![modifier(DiceModifierType::KeepHigh, Some(3))],
        };

        let EvalResult::Rolls(rolls) = eval_expr(&expr) else {
            panic!("Expected rolls");
        };

        assert!(rolls.len() <= 3);
    }

    #[test]
    fn drop_low_modifier_works() {
        let expr = Expr::Dice {
            count: Box::new(num(4)),
            sides: Box::new(num(6)),
            modifiers: vec![modifier(DiceModifierType::DropLow, Some(2))],
        };

        let EvalResult::Rolls(rolls) = eval_expr(&expr) else {
            panic!("Expected rolls");
        };

        assert!(rolls.len() <= 2);
    }

    #[test]
    fn explode_modifier_works() {
        let expr = Expr::Dice {
            count: Box::new(num(2)),
            sides: Box::new(num(3)),
            modifiers: vec![modifier(DiceModifierType::Explode, None)],
        };

        let EvalResult::Rolls(rolls) = eval_expr(&expr) else {
            panic!("Expected rolls");
        };

        assert!(rolls.len() >= 2);
        assert!(rolls.iter().all(|&r| (1..=3).contains(&r)));
    }

    #[test]
    fn test_division_by_zero_panics() {
        let expr = binop(num(4), '/', num(0));
        let result = std::panic::catch_unwind(|| {
            eval_expr(&expr);
        });
        assert!(result.is_err());
    }
}
