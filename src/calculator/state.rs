/// FerrumCalc – Application State
///
/// Contains all types and logic for managing the calculator's runtime state,
/// including mode selection, history, and input handling.

use super::parser;

// ── Calculator Mode ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CalculatorMode {
    Standard,
    Scientific,
    Programmer,
    /// Draft graphing mode: plots `y = f(x)` from the input expression.
    Graph,
}

impl CalculatorMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Standard   => "Standard",
            Self::Scientific => "Scientific",
            Self::Programmer => "Programmer",
            Self::Graph      => "Graph",
        }
    }
}

// ── Programmer Base ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NumBase {
    Hex,
    Dec,
    Oct,
    Bin,
}

impl NumBase {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Hex => "HEX",
            Self::Dec => "DEC",
            Self::Oct => "OCT",
            Self::Bin => "BIN",
        }
    }

    pub fn radix(&self) -> u32 {
        match self {
            Self::Hex => 16,
            Self::Dec => 10,
            Self::Oct => 8,
            Self::Bin => 2,
        }
    }

    /// Returns true if the given character is a valid digit in this base.
    pub fn is_valid_digit(&self, ch: char) -> bool {
        match self {
            Self::Bin => matches!(ch, '0' | '1'),
            Self::Oct => matches!(ch, '0'..='7'),
            Self::Dec => ch.is_ascii_digit(),
            Self::Hex => ch.is_ascii_hexdigit(),
        }
    }
}

// ── History Entry ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoryEntry {
    pub expression: String,
    pub result: String,
}

// ── Angle Mode ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AngleMode {
    Radians,
    Degrees,
}

impl AngleMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Radians => "RAD",
            Self::Degrees => "DEG",
        }
    }
}

// ── Calculator State ─────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CalculatorState {
    /// The current formula/expression being entered
    pub input: String,
    /// The display text for the result
    pub result_display: String,
    /// Whether there was an error in the last evaluation
    pub has_error: bool,
    /// The current calculator mode
    pub mode: CalculatorMode,
    /// Programmer mode number base
    pub base: NumBase,
    /// Angle mode for trigonometric functions
    pub angle_mode: AngleMode,
    /// Calculation history
    pub history: Vec<HistoryEntry>,
    /// Whether the history sidebar is open
    pub history_open: bool,
    /// Whether we just evaluated (next digit input should clear)
    pub just_evaluated: bool,
    /// Use dark theme
    pub dark_mode: bool,
}

impl Default for CalculatorState {
    fn default() -> Self {
        Self {
            input: String::new(),
            result_display: String::from("0"),
            has_error: false,
            mode: CalculatorMode::Standard,
            base: NumBase::Dec,
            angle_mode: AngleMode::Radians,
            history: Vec::new(),
            history_open: false,
            just_evaluated: false,
            dark_mode: true,
        }
    }
}

impl CalculatorState {
    /// Push a character or string to the input buffer.
    pub fn push_input(&mut self, s: &str) {
        if self.just_evaluated {
            // If we just got a result and the user types a digit, start fresh
            if s.chars().next().map_or(false, |c| c.is_ascii_digit() || c == '.') {
                self.input.clear();
                self.result_display = "0".to_string();
                self.has_error = false;
            }
            self.just_evaluated = false;
        }
        self.input.push_str(s);
    }

    /// Delete the last character from the input.
    pub fn backspace(&mut self) {
        self.input.pop();
        if self.input.is_empty() {
            self.result_display = "0".to_string();
            self.has_error = false;
        }
    }

    /// Clear the current input (CE).
    pub fn clear_entry(&mut self) {
        self.input.clear();
        self.result_display = "0".to_string();
        self.has_error = false;
        self.just_evaluated = false;
    }

    /// Clear everything (C).
    pub fn clear_all(&mut self) {
        self.clear_entry();
    }

    /// Evaluate the current expression.
    pub fn evaluate(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }

        let expression = self.input.clone();
        let use_degrees = self.angle_mode == AngleMode::Degrees;

        match parser::evaluate(&expression, use_degrees) {
            Ok(value) => {
                let result_str = if self.mode == CalculatorMode::Programmer {
                    let int_val = value as i64;
                    match self.base {
                        NumBase::Dec => parser::format_result(value),
                        other => parser::to_base_string(int_val, other.radix()),
                    }
                } else {
                    parser::format_result(value)
                };

                self.history.push(HistoryEntry {
                    expression: expression.clone(),
                    result: result_str.clone(),
                });

                self.result_display = result_str;
                self.input = parser::format_result(value);
                self.has_error = false;
                self.just_evaluated = true;
            }
            Err(e) => {
                self.result_display = e;
                self.has_error = true;
                self.just_evaluated = false;
            }
        }
    }

    /// Restore a history entry into the input field.
    pub fn restore_history(&mut self, index: usize) {
        if let Some(entry) = self.history.get(index) {
            self.input = entry.expression.clone();
            self.result_display = entry.result.clone();
            self.has_error = false;
            self.just_evaluated = false;
        }
    }

    /// Clear all history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Toggle the history sidebar.
    pub fn toggle_history(&mut self) {
        self.history_open = !self.history_open;
    }

    /// Insert pasted text into the input buffer.
    ///
    /// Control characters (newlines, tabs, etc.) are stripped so a multi-line
    /// clipboard payload collapses into a single expression. Other characters
    /// are passed through; the parser will reject anything it can't understand
    /// when the expression is evaluated.
    pub fn paste(&mut self, text: &str) {
        let cleaned: String = text.chars().filter(|c| !c.is_control()).collect();
        if !cleaned.is_empty() {
            self.push_input(&cleaned);
        }
    }

    /// Toggle negate the current input.
    pub fn negate(&mut self) {
        if self.input.is_empty() {
            return;
        }
        if self.input.starts_with('-') {
            self.input.remove(0);
        } else {
            self.input.insert(0, '-');
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paste_strips_newlines() {
        let mut state = CalculatorState::default();
        state.paste("2 +\n3\t");
        assert_eq!(state.input, "2 +3");
    }

    #[test]
    fn test_paste_appends_to_existing_input() {
        let mut state = CalculatorState::default();
        state.push_input("1 + ");
        state.paste("2");
        assert_eq!(state.input, "1 + 2");
    }

    #[test]
    fn test_paste_after_evaluation_replaces_when_numeric() {
        let mut state = CalculatorState::default();
        state.push_input("2 + 2");
        state.evaluate();
        assert_eq!(state.input, "4");
        // Pasting a fresh number after a result starts a new expression.
        state.paste("9");
        assert_eq!(state.input, "9");
    }

    #[test]
    fn test_paste_empty_after_cleaning_is_noop() {
        let mut state = CalculatorState::default();
        state.push_input("5");
        state.paste("\n\t\r");
        assert_eq!(state.input, "5");
    }
}
