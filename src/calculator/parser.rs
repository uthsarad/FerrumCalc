/// FerrumCalc – Expression Parser & Evaluator
///
/// A recursive-descent parser that tokenizes and evaluates mathematical
/// expressions with proper operator precedence (PEMDAS), supporting:
///   - Standard arithmetic: +, -, *, /, %
///   - Parentheses
///   - Scientific functions: sin, cos, tan, asin, acos, atan, ln, log, sqrt, abs, ceil, floor
///   - Power operator: ^
///   - Constants: pi, e
///   - Programmer-mode: base prefixes (0b, 0o, 0x), bitwise ops (AND, OR, XOR, NOT, <<, >>)

use std::f64::consts::{E, PI};

// ── Token types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,       // ^
    LParen,
    RParen,
    Comma,
    // Bitwise (programmer mode)
    BitwiseAnd,  // AND / &
    BitwiseOr,   // OR  / |
    BitwiseXor,  // XOR
    BitwiseNot,  // NOT / ~
    ShiftLeft,   // <<
    ShiftRight,  // >>
    // Functions & identifiers
    Ident(String),
    Eof,
}

// ── Lexer ────────────────────────────────────────────────────────────────────

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        self.pos += 1;
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();

            match self.peek() {
                None => {
                    tokens.push(Token::Eof);
                    break;
                }
                Some(ch) => match ch {
                    '+' => { self.advance(); tokens.push(Token::Plus); }
                    '-' => { self.advance(); tokens.push(Token::Minus); }
                    '*' => { self.advance(); tokens.push(Token::Star); }
                    '/' => { self.advance(); tokens.push(Token::Slash); }
                    '%' => { self.advance(); tokens.push(Token::Percent); }
                    '^' => { self.advance(); tokens.push(Token::Caret); }
                    '(' => { self.advance(); tokens.push(Token::LParen); }
                    ')' => { self.advance(); tokens.push(Token::RParen); }
                    ',' => { self.advance(); tokens.push(Token::Comma); }
                    '~' => { self.advance(); tokens.push(Token::BitwiseNot); }
                    '&' => { self.advance(); tokens.push(Token::BitwiseAnd); }
                    '|' => { self.advance(); tokens.push(Token::BitwiseOr); }
                    '<' => {
                        self.advance();
                        if self.peek() == Some('<') {
                            self.advance();
                            tokens.push(Token::ShiftLeft);
                        } else {
                            return Err("Unexpected '<', did you mean '<<'?".to_string());
                        }
                    }
                    '>' => {
                        self.advance();
                        if self.peek() == Some('>') {
                            self.advance();
                            tokens.push(Token::ShiftRight);
                        } else {
                            return Err("Unexpected '>', did you mean '>>'?".to_string());
                        }
                    }
                    _ if ch.is_ascii_digit() || ch == '.' => {
                        tokens.push(self.read_number()?);
                    }
                    _ if ch.is_ascii_alphabetic() || ch == '_' => {
                        let ident = self.read_ident();
                        match ident.to_lowercase().as_str() {
                            "and" => tokens.push(Token::BitwiseAnd),
                            "or"  => tokens.push(Token::BitwiseOr),
                            "xor" => tokens.push(Token::BitwiseXor),
                            "not" => tokens.push(Token::BitwiseNot),
                            _     => tokens.push(Token::Ident(ident.to_lowercase())),
                        }
                    }
                    _ => {
                        return Err(format!("Unexpected character: '{}'", ch));
                    }
                },
            }
        }

        Ok(tokens)
    }

    /// Read a number literal. Supports:
    ///   - Decimal:     123, 3.14
    ///   - Binary:      0b1010
    ///   - Octal:       0o17
    ///   - Hexadecimal: 0xFF
    fn read_number(&mut self) -> Result<Token, String> {
        // Check for base prefix
        if self.peek() == Some('0') {
            let next = self.chars.get(self.pos + 1).copied();
            match next {
                Some('b') | Some('B') => {
                    self.advance(); // '0'
                    self.advance(); // 'b'
                    return self.read_base_number(2);
                }
                Some('o') | Some('O') => {
                    self.advance();
                    self.advance();
                    return self.read_base_number(8);
                }
                Some('x') | Some('X') => {
                    self.advance();
                    self.advance();
                    return self.read_base_number(16);
                }
                _ => {}
            }
        }

        let mut s = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' {
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        s.parse::<f64>()
            .map(Token::Number)
            .map_err(|_| format!("Invalid number: '{}'", s))
    }

    fn read_base_number(&mut self, base: u32) -> Result<Token, String> {
        let mut s = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                if ch != '_' {
                    s.push(ch);
                }
                self.advance();
            } else {
                break;
            }
        }
        if s.is_empty() {
            return Err(format!("Expected digits after base-{} prefix", base));
        }
        i64::from_str_radix(&s, base)
            .map(|v| Token::Number(v as f64))
            .map_err(|_| format!("Invalid base-{} number: '{}'", base, s))
    }

    fn read_ident(&mut self) -> String {
        let mut s = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        s
    }
}

// ── Parser ───────────────────────────────────────────────────────────────────
//
// Grammar (operator-precedence, lowest to highest):
//   expr       → bitwise_or
//   bitwise_or → bitwise_xor ( ("|" | "OR")  bitwise_xor )*
//   bitwise_xor→ bitwise_and ( "XOR" bitwise_and )*
//   bitwise_and→ shift       ( ("&" | "AND") shift )*
//   shift      → add         ( ("<<" | ">>") add )*
//   add        → mul         ( ("+" | "-") mul )*
//   mul        → power       ( ("*" | "/" | "%") power )*
//   power      → unary       ( "^" unary )*          (right-associative)
//   unary      → ("-" | "~" | "NOT") unary | call
//   call       → IDENT "(" args ")" | primary
//   primary    → NUMBER | "(" expr ")" | constant

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    use_degrees: bool,
    /// Value bound to the `x` identifier, used by the graphing mode. `None`
    /// means `x` is undefined and referencing it is an error.
    x: Option<f64>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, use_degrees: bool) -> Self {
        Self { tokens, pos: 0, use_degrees, x: None }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        let tok = self.advance();
        if &tok == expected {
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", expected, tok))
        }
    }

    // ── Entry point ──

    pub fn parse(&mut self) -> Result<f64, String> {
        let result = self.expr()?;
        if *self.peek() != Token::Eof {
            return Err(format!("Unexpected token: {:?}", self.peek().clone()));
        }
        Ok(result)
    }

    fn expr(&mut self) -> Result<f64, String> {
        self.bitwise_or()
    }

    fn bitwise_or(&mut self) -> Result<f64, String> {
        let mut left = self.bitwise_xor()?;
        loop {
            match self.peek() {
                Token::BitwiseOr => {
                    self.advance();
                    let right = self.bitwise_xor()?;
                    left = ((left as i64) | (right as i64)) as f64;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn bitwise_xor(&mut self) -> Result<f64, String> {
        let mut left = self.bitwise_and()?;
        loop {
            match self.peek() {
                Token::BitwiseXor => {
                    self.advance();
                    let right = self.bitwise_and()?;
                    left = ((left as i64) ^ (right as i64)) as f64;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn bitwise_and(&mut self) -> Result<f64, String> {
        let mut left = self.shift()?;
        loop {
            match self.peek() {
                Token::BitwiseAnd => {
                    self.advance();
                    let right = self.shift()?;
                    left = ((left as i64) & (right as i64)) as f64;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn shift(&mut self) -> Result<f64, String> {
        let mut left = self.add()?;
        loop {
            match self.peek() {
                Token::ShiftLeft => {
                    self.advance();
                    let right = self.add()?;
                    left = ((left as i64) << (right as i64)) as f64;
                }
                Token::ShiftRight => {
                    self.advance();
                    let right = self.add()?;
                    left = ((left as i64) >> (right as i64)) as f64;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn add(&mut self) -> Result<f64, String> {
        let mut left = self.mul()?;
        loop {
            match self.peek() {
                Token::Plus => {
                    self.advance();
                    let right = self.mul()?;
                    left += right;
                }
                Token::Minus => {
                    self.advance();
                    let right = self.mul()?;
                    left -= right;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn mul(&mut self) -> Result<f64, String> {
        let mut left = self.power()?;
        loop {
            match self.peek() {
                Token::Star => {
                    self.advance();
                    let right = self.power()?;
                    left *= right;
                }
                Token::Slash => {
                    self.advance();
                    let right = self.power()?;
                    if right == 0.0 {
                        return Err("Division by zero".to_string());
                    }
                    left /= right;
                }
                Token::Percent => {
                    self.advance();
                    let right = self.power()?;
                    if right == 0.0 {
                        return Err("Modulo by zero".to_string());
                    }
                    left %= right;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn power(&mut self) -> Result<f64, String> {
        let base = self.unary()?;
        if *self.peek() == Token::Caret {
            self.advance();
            // Right-associative: parse power recursively
            let exp = self.power()?;
            Ok(base.powf(exp))
        } else {
            Ok(base)
        }
    }

    fn unary(&mut self) -> Result<f64, String> {
        match self.peek() {
            Token::Minus => {
                self.advance();
                let val = self.unary()?;
                Ok(-val)
            }
            Token::BitwiseNot => {
                self.advance();
                let val = self.unary()?;
                Ok(!(val as i64) as f64)
            }
            _ => self.call(),
        }
    }

    fn call(&mut self) -> Result<f64, String> {
        if let Token::Ident(name) = self.peek().clone() {
            // Check if this is a function call (followed by '(')
            if self.tokens.get(self.pos + 1) == Some(&Token::LParen) {
                self.advance(); // consume ident
                self.advance(); // consume '('
                let mut args = Vec::new();
                if *self.peek() != Token::RParen {
                    args.push(self.expr()?);
                    while *self.peek() == Token::Comma {
                        self.advance();
                        args.push(self.expr()?);
                    }
                }
                self.expect(&Token::RParen)?;
                return self.eval_function(&name, &args);
            }
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<f64, String> {
        match self.peek().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(n)
            }
            Token::LParen => {
                self.advance();
                let val = self.expr()?;
                self.expect(&Token::RParen)?;
                Ok(val)
            }
            Token::Ident(name) => {
                self.advance();
                // Constants
                match name.as_str() {
                    "pi" => Ok(PI),
                    "e"  => Ok(E),
                    "x"  => self.x.ok_or_else(|| "Unknown identifier: 'x'".to_string()),
                    _    => Err(format!("Unknown identifier: '{}'", name)),
                }
            }
            other => Err(format!("Unexpected token: {:?}", other)),
        }
    }

    fn eval_function(&self, name: &str, args: &[f64]) -> Result<f64, String> {
        match name {
            "sin"   => { Self::expect_args(name, args, 1)?; Ok(if self.use_degrees { args[0].to_radians().sin() } else { args[0].sin() }) }
            "cos"   => { Self::expect_args(name, args, 1)?; Ok(if self.use_degrees { args[0].to_radians().cos() } else { args[0].cos() }) }
            "tan"   => { Self::expect_args(name, args, 1)?; Ok(if self.use_degrees { args[0].to_radians().tan() } else { args[0].tan() }) }
            "asin"  => { Self::expect_args(name, args, 1)?; Ok(if self.use_degrees { args[0].asin().to_degrees() } else { args[0].asin() }) }
            "acos"  => { Self::expect_args(name, args, 1)?; Ok(if self.use_degrees { args[0].acos().to_degrees() } else { args[0].acos() }) }
            "atan"  => { Self::expect_args(name, args, 1)?; Ok(if self.use_degrees { args[0].atan().to_degrees() } else { args[0].atan() }) }
            "sinh"  => { Self::expect_args(name, args, 1)?; Ok(args[0].sinh()) }
            "cosh"  => { Self::expect_args(name, args, 1)?; Ok(args[0].cosh()) }
            "tanh"  => { Self::expect_args(name, args, 1)?; Ok(args[0].tanh()) }
            "ln"    => { Self::expect_args(name, args, 1)?; Ok(args[0].ln()) }
            "log"   => { Self::expect_args(name, args, 1)?; Ok(args[0].log10()) }
            "log2"  => { Self::expect_args(name, args, 1)?; Ok(args[0].log2()) }
            "sqrt"  => { Self::expect_args(name, args, 1)?; Ok(args[0].sqrt()) }
            "cbrt"  => { Self::expect_args(name, args, 1)?; Ok(args[0].cbrt()) }
            "abs"   => { Self::expect_args(name, args, 1)?; Ok(args[0].abs()) }
            "ceil"  => { Self::expect_args(name, args, 1)?; Ok(args[0].ceil()) }
            "floor" => { Self::expect_args(name, args, 1)?; Ok(args[0].floor()) }
            "round" => { Self::expect_args(name, args, 1)?; Ok(args[0].round()) }
            "exp"   => { Self::expect_args(name, args, 1)?; Ok(args[0].exp()) }
            "fact"  => { Self::expect_args(name, args, 1)?; Self::factorial(args[0]) }
            "pow"   => { Self::expect_args(name, args, 2)?; Ok(args[0].powf(args[1])) }
            "max"   => { Self::expect_args(name, args, 2)?; Ok(args[0].max(args[1])) }
            "min"   => { Self::expect_args(name, args, 2)?; Ok(args[0].min(args[1])) }
            _ => Err(format!("Unknown function: '{}'", name)),
        }
    }

    fn expect_args(name: &str, args: &[f64], expected: usize) -> Result<(), String> {
        if args.len() != expected {
            Err(format!(
                "{}() expects {} argument(s), got {}",
                name,
                expected,
                args.len()
            ))
        } else {
            Ok(())
        }
    }

    fn factorial(n: f64) -> Result<f64, String> {
        if n < 0.0 || n.fract() != 0.0 {
            return Err("Factorial requires a non-negative integer".to_string());
        }
        let n = n as u64;
        if n > 20 {
            return Err("Factorial argument too large (max 20)".to_string());
        }
        let mut result: u64 = 1;
        for i in 2..=n {
            result *= i;
        }
        Ok(result as f64)
    }
}

// ── Public evaluate function ─────────────────────────────────────────────────

/// Evaluate a mathematical expression string and return the result.
pub fn evaluate(input: &str, use_degrees: bool) -> Result<f64, String> {
    if input.trim().is_empty() {
        return Err("Empty expression".to_string());
    }
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens, use_degrees);
    parser.parse()
}

/// Evaluate an expression with the variable `x` bound to a value.
///
/// Used by the graphing mode to sample `y = f(x)` across a domain. Identical to
/// [`evaluate`] except that references to `x` resolve to `x_value`.
pub fn evaluate_at(input: &str, use_degrees: bool, x_value: f64) -> Result<f64, String> {
    if input.trim().is_empty() {
        return Err("Empty expression".to_string());
    }
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens, use_degrees);
    parser.x = Some(x_value);
    parser.parse()
}

/// Format a result for display, removing unnecessary trailing zeros.
pub fn format_result(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 { "∞".to_string() } else { "-∞".to_string() };
    }
    // If it's a whole number, display without decimal
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        // Use up to 12 significant digits
        let s = format!("{:.12}", value);
        // Trim trailing zeros
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

/// Convert an integer value to a string in the given base.
pub fn to_base_string(value: i64, base: u32) -> String {
    match base {
        2  => format!("0b{:b}", value),
        8  => format!("0o{:o}", value),
        16 => format!("0x{:X}", value),
        _  => format!("{}", value),
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(input: &str) -> f64 {
        evaluate(input, false).unwrap()
    }

    fn eval_err(input: &str) -> String {
        evaluate(input, false).unwrap_err()
    }

    // Basic arithmetic
    #[test]
    fn test_addition() {
        assert_eq!(eval("2 + 3"), 5.0);
    }

    #[test]
    fn test_subtraction() {
        assert_eq!(eval("10 - 4"), 6.0);
    }

    #[test]
    fn test_multiplication() {
        assert_eq!(eval("6 * 7"), 42.0);
    }

    #[test]
    fn test_division() {
        assert_eq!(eval("15 / 3"), 5.0);
    }

    #[test]
    fn test_modulo() {
        assert_eq!(eval("17 % 5"), 2.0);
    }

    // Operator precedence
    #[test]
    fn test_precedence_mul_over_add() {
        assert_eq!(eval("2 + 3 * 4"), 14.0);
    }

    #[test]
    fn test_parentheses_override() {
        assert_eq!(eval("(2 + 3) * 4"), 20.0);
    }

    #[test]
    fn test_nested_parentheses() {
        assert_eq!(eval("((2 + 3) * (4 - 1))"), 15.0);
    }

    // Power (right-associative)
    #[test]
    fn test_power() {
        assert_eq!(eval("2 ^ 10"), 1024.0);
    }

    #[test]
    fn test_power_right_assoc() {
        // 2^3^2 = 2^(3^2) = 2^9 = 512
        assert_eq!(eval("2 ^ 3 ^ 2"), 512.0);
    }

    // Unary minus
    #[test]
    fn test_unary_minus() {
        assert_eq!(eval("-5"), -5.0);
    }

    #[test]
    fn test_unary_minus_in_expr() {
        assert_eq!(eval("3 + -2"), 1.0);
    }

    // Scientific functions
    #[test]
    fn test_sqrt() {
        assert_eq!(eval("sqrt(16)"), 4.0);
    }

    #[test]
    fn test_sin_pi_over_2() {
        assert!((eval("sin(pi / 2)") - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cos_zero() {
        assert!((eval("cos(0)") - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_ln_e() {
        assert!((eval("ln(e)") - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_log10() {
        assert!((eval("log(100)") - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_factorial() {
        assert_eq!(eval("fact(5)"), 120.0);
    }

    #[test]
    fn test_abs() {
        assert_eq!(eval("abs(-42)"), 42.0);
    }

    // Multi-argument functions
    #[test]
    fn test_pow_function() {
        assert_eq!(eval("pow(2, 8)"), 256.0);
    }

    #[test]
    fn test_max() {
        assert_eq!(eval("max(3, 7)"), 7.0);
    }

    // Programmer mode – base literals
    #[test]
    fn test_binary_literal() {
        assert_eq!(eval("0b1010"), 10.0);
    }

    #[test]
    fn test_octal_literal() {
        assert_eq!(eval("0o17"), 15.0);
    }

    #[test]
    fn test_hex_literal() {
        assert_eq!(eval("0xFF"), 255.0);
    }

    // Bitwise operations
    #[test]
    fn test_bitwise_and() {
        assert_eq!(eval("0b1100 & 0b1010"), 8.0); // 1000 = 8
    }

    #[test]
    fn test_bitwise_or() {
        assert_eq!(eval("0b1100 | 0b1010"), 14.0); // 1110 = 14
    }

    #[test]
    fn test_bitwise_xor() {
        assert_eq!(eval("12 XOR 10"), 6.0);
    }

    #[test]
    fn test_bitwise_not() {
        // NOT 0 should be -1 in two's complement
        assert_eq!(eval("~0"), -1.0);
    }

    #[test]
    fn test_shift_left() {
        assert_eq!(eval("1 << 4"), 16.0);
    }

    #[test]
    fn test_shift_right() {
        assert_eq!(eval("16 >> 2"), 4.0);
    }

    // Error cases
    #[test]
    fn test_division_by_zero() {
        assert!(eval_err("1 / 0").contains("Division by zero"));
    }

    #[test]
    fn test_unknown_function() {
        assert!(eval_err("foo(1)").contains("Unknown function"));
    }

    #[test]
    fn test_empty_expression() {
        assert!(eval_err("").contains("Empty expression"));
    }

    // Complex expressions
    #[test]
    fn test_complex_expression() {
        // (3 + 4) * 2 - sqrt(16) + 1 = 7*2 - 4 + 1 = 11
        assert_eq!(eval("(3 + 4) * 2 - sqrt(16) + 1"), 11.0);
    }

    #[test]
    fn test_nested_functions() {
        assert_eq!(eval("abs(sqrt(144) - 15)"), 3.0);
    }

    // Format result
    #[test]
    fn test_format_integer() {
        assert_eq!(format_result(42.0), "42");
    }

    #[test]
    fn test_format_decimal() {
        assert_eq!(format_result(3.14), "3.14");
    }

    // Graphing variable (x)
    #[test]
    fn test_variable_x_linear() {
        assert_eq!(evaluate_at("2*x + 1", false, 3.0).unwrap(), 7.0);
    }

    #[test]
    fn test_variable_x_in_function() {
        assert_eq!(evaluate_at("x^2", false, 4.0).unwrap(), 16.0);
    }

    #[test]
    fn test_variable_x_undefined_without_binding() {
        assert!(eval_err("x + 1").contains("Unknown identifier: 'x'"));
    }

    // Base conversion
    #[test]
    fn test_to_binary_string() {
        assert_eq!(to_base_string(10, 2), "0b1010");
    }

    #[test]
    fn test_to_hex_string() {
        assert_eq!(to_base_string(255, 16), "0xFF");
    }

    // Degree mode tests
    #[test]
    fn test_trig_degrees() {
        assert!((evaluate("sin(30)", true).unwrap() - 0.5).abs() < 1e-10);
        assert!((evaluate("cos(60)", true).unwrap() - 0.5).abs() < 1e-10);
        assert!((evaluate("tan(45)", true).unwrap() - 1.0).abs() < 1e-10);
        assert!((evaluate("asin(0.5)", true).unwrap() - 30.0).abs() < 1e-10);
        assert!((evaluate("acos(0.5)", true).unwrap() - 60.0).abs() < 1e-10);
        assert!((evaluate("atan(1.0)", true).unwrap() - 45.0).abs() < 1e-10);
    }
}
