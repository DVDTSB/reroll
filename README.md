# 🎲 roll

A powerful and flexible command-line dice roller for tabletop games and probability testing.

Supports:
- Standard dice expressions like `d8`,`3d6`, `4d6kh3`, and `2d10dl1`
- Arithmetic operations: `+`, `-`, `*`, `/`
- Repetition syntax: `3(1d6+2)` rolls `1d6+2` three times
- Modifiers: explode (`!`), keep/drop high/low (`k/kh`, `kl`, `d/dh`, `dl`)

## 🧾 Usage

```sh
roll "4d6kh3 + 2"
roll "3(2d8+1)"
roll "1d20 + 5" 
````

Use quotes around expressions to avoid shell interference.

## 🔍 Options

* `-v` or `--verbose`: Show all final rolls.
* `-h` or `--help`: Print help and usage info.

## ✨ Features

* Full support for nested expressions with correct order of operations
* Exploding dice (e.g., `1d6!`) with custom thresholds (e.g., `1d6!5`)
* Deterministic expression parsing using [pest](https://pest.rs/)

## 🧪 Example Output

```sh
$ roll "4d6kh3"
14

$ roll -v "4d6kh3"
[6, 4, 4]
```

## 🔧 Build

```sh
cargo build --release
```

## ✅ Tests

```sh
cargo test
```

## 📁 File Structure

* `src/parser.rs`: Expression parsing (via Pest)
* `src/eval.rs`: Expression evaluation and dice logic
* `src/main.rs`: CLI frontend

## 📜 Syntax Reference

| Example     | Meaning                      |
| ----------- | ---------------------------- |
| `3d6`       | Roll 3 six-sided dice        |
| `4d6kh3`    | Roll 4d6, keep the highest 3 |
| `2d8dl1`    | Roll 2d8, drop the lowest 1  |
| `1d6!`      | Exploding dice on max roll   |
| `3(1d6+2)`  | Roll `1d6+2` three times     |
| `(2d6+1)*2` | Roll and apply arithmetic    |

---

Made with 🍀 for dice goblins and probability nerds.
