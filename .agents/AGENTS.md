# FerrumCalc — Agent Guidelines

## Project Overview

FerrumCalc is a native desktop calculator written in **Rust** using **egui** (via **eframe 0.31**). It supports three modes: Standard, Scientific, and Programmer. It includes a custom recursive-descent expression parser, full keyboard input, a collapsible history sidebar, and a dark/light theme with Fluent-inspired aesthetics.

## Architecture

```
src/
├── main.rs                   # Entry point: eframe window config, launches FerrumCalcApp
├── icon.rs                   # Procedurally-generated window icon (raw RGBA, no image asset)
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

## Persistence

- The `persistence` feature is enabled on `eframe` in `Cargo.toml`. This pulls in `ron` (serialization) and `arboard` (clipboard).
- `CalculatorState` derives `serde::Serialize`/`Deserialize`. The entire state — including history and preferences — is the persisted unit.
- **Save**: `FerrumCalcApp::save` calls `eframe::set_value(storage, eframe::APP_KEY, &self.state)`. eframe invokes `save` periodically and on exit.
- **Load**: `FerrumCalcApp::new` reads `cc.storage` via `eframe::get_value::<CalculatorState>(storage, eframe::APP_KEY)`, falling back to `Default` on first launch or unreadable data. **The working expression and result are cleared on load** (`input`, `result_display`, `has_error`, `just_evaluated`) so the calculator always opens fresh — only preferences (mode, base, angle, theme) and history are restored. If you want a launch to also forget history, clear `state.history` in that same `.map(...)` closure.
- **Clipboard**: copy is deferred until *after* the `ctx.input(..)` closure returns — calling `ctx` methods from inside that closure can deadlock. Paste arrives as `egui::Event::Paste` and is routed through `CalculatorState::paste`, which strips control characters.

## Window Icon

- `src/icon.rs` builds the window/taskbar icon at runtime as a raw 256×256 RGBA buffer (`egui::IconData`) and `main.rs` passes it via `ViewportBuilder::with_icon`. There is **no** image file and **no** PNG decoder dependency.
- The motif is a flat calculator (rounded indigo square + light display bar + 3×3 keypad, right column orange) drawn with signed-distance-field coverage for lightly anti-aliased edges.
- To tweak the design, edit the color constants and the `col_x`/`row_y` layout arrays in `app_icon`. The unit tests assert dimensions, a transparent corner, an opaque indigo body, and the orange operator column — update them if you change the layout.

## Keypad Layout & Scaling

- Both keypad renderers (`render_button_grid`, `render_button_grid_with_base`) size buttons to fill the panel's *remaining* height: `btn_height = (available_height − (rows−1)·spacing) / rows`, clamped to a `[min, max]` range.
- Two things must stay in sync or the grid overflows the window bottom: (1) `ui.spacing_mut().item_spacing.y` is pinned to the same `spacing` used in the height math, and (2) `available_height` subtracts one extra `spacing` to account for the leading gap egui inserts before the first row.
- Keep the `min` clamp low enough that the tallest keypad (Programmer, 8 rows) still fits at the minimum window size (320×480). Raising it risks reintroducing bottom overflow.

## Graph Mode (draft)

> Marked **draft** in the UI (a "DRAFT" badge in the header). It is functional but intentionally minimal — see the polish items under "Potential Improvements".

- A fourth `CalculatorMode::Graph` variant. Selectable from the toolbar (and the compact history-open ComboBox).
- **Variable `x`**: the parser resolves the identifier `x` via `parser::evaluate_at(input, use_degrees, x)`. Plain `evaluate` leaves `x` undefined (errors), so non-graph modes are unaffected.
- **Live plotting**: there is no "=" in graph mode; the plot re-samples `self.state.input` every frame. `render_graph_canvas` evaluates `y = f(x)` once per horizontal pixel over a fixed `[-10, 10] × [-10, 10]` domain, breaking the line across large jumps (asymptotes) and clipping to the plot rect via `ui.painter_at`.
- **Layout**: graph mode bypasses `render_display`/`render_keypad` and uses `render_graph_display` + `render_graph_canvas` + `render_graph_keypad`. The canvas takes ~half the remaining height (clamped) so the keypad still fits.
- **Typing**: letters are accepted from the keyboard in Scientific and Graph modes (so you can type `sin(x)`); they remain hex-only in Programmer mode and ignored in Standard. `Enter` is a no-op in graph mode.
- The expression is shared across modes (it's the same `input` buffer), so switching from Graph back to Standard and pressing `=` evaluates whatever is typed (with `x` undefined).

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
- Persistent state: history and preferences saved to disk via eframe storage (see "Persistence" below)
- Copy/paste support (Ctrl/Cmd+C copies the result, Ctrl/Cmd+V pastes a sanitized expression)
- Procedurally-generated window/taskbar icon (see "Window Icon" below)
- Keypad sizes to fit the panel exactly in every mode, so the taller Scientific (7 rows) and Programmer (8 rows) keypads never overflow the window bottom
- Fresh-start on launch: saved preferences and history are restored, but the working expression/result is always cleared (see "Persistence")
- Graph mode (**draft**): plots `y = f(x)` from the input expression (see "Graph Mode (draft)" below)
- 51 unit tests all passing

### 🔲 Potential Improvements
- Graph mode polish: pan/zoom, auto-fit y-range, trace/cursor readout, multiple plotted functions
- Keyboard shortcut hints or tooltips on buttons
- Animation/transition effects when switching modes
- Inverse trig/hyperbolic function panel (asin, acos, atan, etc. — parser supports them, UI buttons not yet exposed)
- Memory functions (M+, M-, MR, MC)
- Unit conversion mode
- Release build optimizations and Windows installer (MSI/NSIS)

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `cargo build` says exit code 1 but shows "Finished" | PowerShell stderr issue. Check `$LASTEXITCODE` — it should be 0. |
| Font not found / compile error about `include_bytes!` | Ensure `assets/JetBrainsMono-Regular.ttf` exists relative to `Cargo.toml`. |
| `CornerRadius` / `Rounding` type errors | You're likely using old egui 0.28 API. See "Critical API Notes" section above. |
| Window doesn't open on Windows | Ensure `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` is the first line in `main.rs` (before doc comments). In debug mode, a console window is expected. |
