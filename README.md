# FerrumCalc 🧮

A modern, high-performance native desktop calculator built with **Rust** and **egui**.

![Rust](https://img.shields.io/badge/Rust-stable-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- **Three Calculator Modes**
  - **Standard** — Basic arithmetic with a clean 4×5 keypad
  - **Scientific** — Trigonometry, logarithms, powers, factorial, constants (π, e), RAD/DEG toggle
  - **Programmer** — Hex/Dec/Oct/Bin base switching, bitwise operations (AND, OR, XOR, NOT, shifts), live base conversions

- **Smart Expression Evaluation**
  - Formula-based input — type full expressions like `2 + 3 * 4`
  - Proper operator precedence (PEMDAS)
  - Parentheses support with nested grouping
  - Base-prefixed literals: `0b1010`, `0o17`, `0xFF`

- **Modern UI**
  - Fluent-inspired dark and light themes
  - JetBrains Mono display font
  - Smooth hover effects and accent-colored equals button
  - Responsive button grid that adapts to window size

- **Keyboard Support** — Type naturally with number keys, operators, Enter to evaluate, Escape to clear

- **History Sidebar** — Collapsible panel showing past calculations, click any entry to restore it

## Quick Start

```bash
# Clone and run
git clone <repo-url>
cd FerrumCalc
cargo run
```

## Requirements

- Rust stable toolchain (1.70+)
- Windows / macOS / Linux (cross-platform via egui)

## Build

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test
```

## Project Structure

```
FerrumCalc/
├── Cargo.toml
├── assets/
│   └── JetBrainsMono-Regular.ttf
├── src/
│   ├── main.rs              # Entry point
│   ├── ui.rs                # GUI rendering & theming
│   └── calculator/
│       ├── mod.rs            # Module declarations
│       ├── parser.rs         # Expression parser & evaluator (+ 39 tests)
│       └── state.rs          # Application state management
└── .agents/
    └── AGENTS.md             # Agent development guidelines
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `0-9`, `.` | Input digits |
| `+`, `-`, `*`, `/`, `%`, `^` | Operators |
| `(`, `)` | Parentheses |
| `Enter` | Evaluate expression |
| `Backspace` | Delete last character |
| `Escape` / `Delete` | Clear all |
| `A-F` | Hex digits (Programmer mode) |

## License

MIT
