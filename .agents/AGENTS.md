# FerrumCalc — Agent Guidelines

## Project Overview

FerrumCalc is a native desktop calculator written in **Rust** using **egui** (via **eframe 0.31**). It supports three modes: Standard, Scientific, and Programmer. It includes a custom recursive-descent expression parser, full keyboard input, a collapsible history sidebar, and a dark/light theme with Fluent-inspired aesthetics.

## Architecture

```
src/
├── main.rs                   # Entry point: eframe window config, launches FerrumCalcApp
├── ui.rs                     # All GUI rendering, theming, keyboard handling (~700 lines)
└── calculator/
    ├── mod.rs                # Module re-exports
    ├── parser.rs             # Lexer → Tokenizer → Recursive-descent Parser → Evaluator (native angle mode support)
    └── state.rs              # App state: modes, history, input buffer, evaluation logic
```

### Module Responsibilities

| Module | Responsibility |
|--------|---------------|
| `parser.rs` | Pure computation: tokenize input string → parse with operator precedence → evaluate to `f64` (with native degree/radian mode support). No UI dependencies. Contains all unit tests. |
| `state.rs` | Manages `CalculatorState`: input buffer, result display, mode selection, history list, angle mode, base selection. Calls `parser::evaluate(input, use_degrees)`. |
| `ui.rs` | Renders the full GUI. Contains `FerrumCalcApp` (implements `eframe::App`). Owns a `CalculatorState`. Handles button clicks, keyboard events, theming. |
| `main.rs` | Thin entry point. Configures `eframe::NativeOptions` and runs the app. |

### Data Flow

```
User Input (click or key) → ui.rs handles event → state.rs mutates CalculatorState
    → state.evaluate() calls parser::evaluate(input, use_degrees) → result stored back in state
    → ui.rs reads state to render display
```

## Critical Mathematical & Parser Notes

> **These are essential principles for extending the calculator's mathematical engine.**

- **Native Angle Mode**: The parser handles the `use_degrees` flag natively inside `Parser::eval_function`:
  - **Trigonometric functions** (`sin`, `cos`, `tan`): If `use_degrees` is true, convert the argument from degrees to radians using `args[0].to_radians()` before computing.
  - **Inverse trigonometric functions** (`asin`, `acos`, `atan`): If `use_degrees` is true, convert the computed radian result to degrees using `result.to_degrees()`.
  - **Hyperbolic functions** (`sinh`, `cosh`, `tanh`): These are angle-mode independent and always take real/radian values. Do not apply degree conversions.
- **No String Preprocessing**: Do not perform string-based search-and-replace to wrap trig arguments in `state.rs`. This was a major source of infinite loops during evaluation and failed to support inverse trigonometric functions. Always delegate angle-mode conversions to the parser.

## Critical API Notes (eframe 0.31)

> **These are the most common pitfalls when modifying UI code.**

- **`Rounding` is renamed to `CornerRadius`**: Use `egui::CornerRadius::same(12)` not `egui::Rounding::same(12.0)`.
- **`CornerRadius::same()` takes `u8`**, not `f32`. Use integer literals: `same(8)` not `same(8.0)`.
- **`Margin::same()` and `Margin::symmetric()` take `i8`**, not `f32`. Use integer literals.
- **`.rounding()` method is renamed to `.corner_radius()`** on `Button`, `Frame`, etc.
- **`visuals.window_rounding`** → **`visuals.window_corner_radius`**.
- **`widgets.*.rounding`** field → **`widgets.*.corner_radius`** field (it's a direct field, not a method call on `WidgetVisuals`).
- **`#![cfg_attr(...)]`** inner attributes must appear before any doc comments in `main.rs`.
- **egui is re-exported by eframe**: Access via `eframe::egui` or `use eframe::egui;` — no need to add `egui` as a separate dependency.

## Build & Test Commands

```powershell
# Build (debug)
cargo build

# Run
cargo run

# Run tests (40 tests in parser.rs)
cargo test

# Build release
cargo build --release
```

> **Note:** On PowerShell, `cargo build 2>&1` may falsely report exit code 1 because cargo writes progress to stderr. Use `cargo build; echo $LASTEXITCODE` to verify.

## Coding Conventions

- **Comments**: Use `///` doc comments on public items. Use `//` for inline explanations.
- **Section dividers**: Files use `// ── Section Name ──` banner comments to separate logical sections.
- **Error handling**: The parser returns `Result<f64, String>` with human-readable error messages.
- **Naming**: Types use PascalCase, functions use snake_case, constants use SCREAMING_SNAKE_CASE.
- **Testing**: All parser/evaluator tests live in `parser.rs` under `#[cfg(test)] mod tests`. Test both success and error paths, including angle modes.
- **UI button definitions**: Buttons are defined as `(&str, ButtonKind)` tuples in grid arrays. Use `ButtonKind` enum to classify behavior. Display label can differ from the action label (e.g., `"C_hex"` displays as `"C"` but acts as hex digit C).

## Assets

- `assets/JetBrainsMono-Regular.ttf` — Embedded at compile time via `include_bytes!()` in `ui.rs`. Used for the calculator display font.

## Current Feature Status

### ✅ Complete
- Standard mode (digits, basic arithmetic, clear, backspace, negate, equals)
- Scientific mode (trig, hyperbolic, log/ln, sqrt/cbrt, power, factorial, abs, ceil, floor, round, exp, constants π/e, parentheses, RAD/DEG toggle with native angle-mode evaluation)
- Programmer mode (hex/dec/oct/bin base selector, hex digit buttons A-F, bitwise AND/OR/XOR/NOT/shifts, base prefix literals 0b/0o/0x, live base conversions in display, digit disabling per base)
- Formula-based evaluation with proper operator precedence (PEMDAS + bitwise)
- Full keyboard input
- Collapsible history sidebar (click to restore, clear button)
- Dark/light theme toggle with Fluent-inspired color palette
- 40 unit tests all passing

### 🔲 Potential Improvements
- Persist history and preferences to disk (serde is already a dependency)
- Copy/paste support (Ctrl+C to copy result, Ctrl+V to paste expression)
- Keyboard shortcut hints or tooltips on buttons
- Animation/transition effects when switching modes
- Custom window icon
- Inverse trig/hyperbolic function panel (asin, acos, atan, etc. — parser supports them, UI buttons not yet exposed)
- Memory functions (M+, M-, MR, MC)
- Unit conversion mode
- Responsive layout that adapts button sizes more fluidly
- Release build optimizations and Windows installer (MSI/NSIS)

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `cargo build` says exit code 1 but shows "Finished" | PowerShell stderr issue. Check `$LASTEXITCODE` — it should be 0. |
| Font not found / compile error about `include_bytes!` | Ensure `assets/JetBrainsMono-Regular.ttf` exists relative to `Cargo.toml`. |
| `CornerRadius` / `Rounding` type errors | You're likely using old egui 0.28 API. See "Critical API Notes" section above. |
| Window doesn't open on Windows | Ensure `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` is the first line in `main.rs` (before doc comments). In debug mode, a console window is expected. |
