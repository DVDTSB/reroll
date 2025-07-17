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
