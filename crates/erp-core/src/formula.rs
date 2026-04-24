use std::collections::HashMap;
use thiserror::Error;

/// Errors from formula evaluation.
#[derive(Debug, Error)]
pub enum FormulaError {
    #[error("Unknown variable: ${{{0}}}")]
    UnknownVariable(String),
    #[error("Unexpected token: '{0}' at position {1}")]
    UnexpectedToken(String, usize),
    #[error("Unexpected end of formula")]
    UnexpectedEnd,
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Unmatched parenthesis")]
    UnmatchedParen,
    #[error("Unknown function: {0}")]
    UnknownFunction(String),
    #[error("Invalid number of arguments for {0}: expected {1}, got {2}")]
    InvalidArgCount(String, usize, usize),
}

/// Evaluate a parametric formula string against a parameter map.
///
/// Supports: +, -, *, /, parentheses, `${variable}` substitution,
/// and built-in functions: max(), min(), round(), abs(), sqrt()
///
/// Uses a recursive descent parser. No eval() or unsafe code.
pub fn evaluate_formula(
    formula: &str,
    params: &HashMap<String, f64>,
) -> Result<f64, FormulaError> {
    let tokens = tokenize(formula, params)?;
    let mut parser = Parser::new(&tokens);
    let result = parser.parse_expression()?;
    if parser.pos < parser.tokens.len() {
        return Err(FormulaError::UnexpectedToken(
            format!("{:?}", parser.tokens[parser.pos]),
            parser.pos,
        ));
    }
    Ok(result)
}

#[derive(Debug, Clone)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Mul,
    Div,
    LParen,
    RParen,
    Comma,
    Func(String),
}

/// Tokenize the formula string, substituting variables inline.
fn tokenize(formula: &str, params: &HashMap<String, f64>) -> Result<Vec<Token>, FormulaError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = formula.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\n' | '\r' => {
                i += 1;
            }
            '+' => {
                tokens.push(Token::Plus);
                i += 1;
            }
            '-' => {
                tokens.push(Token::Minus);
                i += 1;
            }
            '*' => {
                tokens.push(Token::Mul);
                i += 1;
            }
            '/' => {
                tokens.push(Token::Div);
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                i += 1;
            }
            '$' => {
                // Variable substitution: ${name}
                if i + 1 < chars.len() && chars[i + 1] == '{' {
                    let start = i + 2;
                    let mut end = start;
                    while end < chars.len() && chars[end] != '}' {
                        end += 1;
                    }
                    if end >= chars.len() {
                        return Err(FormulaError::UnexpectedEnd);
                    }
                    let var_name: String = chars[start..end].iter().collect();
                    let value = params
                        .get(&var_name)
                        .ok_or_else(|| FormulaError::UnknownVariable(var_name))?;
                    tokens.push(Token::Number(*value));
                    i = end + 1;
                } else {
                    return Err(FormulaError::UnexpectedToken("$".into(), i));
                }
            }
            c if c.is_ascii_digit() || c == '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let num_str: String = chars[start..i].iter().collect();
                let num: f64 = num_str
                    .parse()
                    .map_err(|_| FormulaError::UnexpectedToken(num_str, start))?;
                tokens.push(Token::Number(num));
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let name: String = chars[start..i].iter().collect();
                tokens.push(Token::Func(name));
            }
            other => {
                return Err(FormulaError::UnexpectedToken(other.to_string(), i));
            }
        }
    }

    Ok(tokens)
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        self.pos += 1;
        tok
    }

    /// expression = term (('+' | '-') term)*
    fn parse_expression(&mut self) -> Result<f64, FormulaError> {
        let mut left = self.parse_term()?;
        while let Some(tok) = self.peek() {
            match tok {
                Token::Plus => {
                    self.advance();
                    left += self.parse_term()?;
                }
                Token::Minus => {
                    self.advance();
                    left -= self.parse_term()?;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// term = unary (('*' | '/') unary)*
    fn parse_term(&mut self) -> Result<f64, FormulaError> {
        let mut left = self.parse_unary()?;
        while let Some(tok) = self.peek() {
            match tok {
                Token::Mul => {
                    self.advance();
                    left *= self.parse_unary()?;
                }
                Token::Div => {
                    self.advance();
                    let right = self.parse_unary()?;
                    if right == 0.0 {
                        return Err(FormulaError::DivisionByZero);
                    }
                    left /= right;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// unary = ('-')? primary
    fn parse_unary(&mut self) -> Result<f64, FormulaError> {
        if let Some(Token::Minus) = self.peek() {
            self.advance();
            let val = self.parse_primary()?;
            Ok(-val)
        } else {
            self.parse_primary()
        }
    }

    /// primary = Number | Func '(' args ')' | '(' expression ')'
    fn parse_primary(&mut self) -> Result<f64, FormulaError> {
        match self.advance() {
            Some(Token::Number(n)) => Ok(*n),
            Some(Token::Func(name)) => {
                let name = name.clone();
                // Expect '('
                match self.advance() {
                    Some(Token::LParen) => {}
                    _ => return Err(FormulaError::UnexpectedToken(name, self.pos)),
                }

                // Parse arguments
                let mut args = Vec::new();
                if !matches!(self.peek(), Some(Token::RParen)) {
                    args.push(self.parse_expression()?);
                    while matches!(self.peek(), Some(Token::Comma)) {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                }

                // Expect ')'
                match self.advance() {
                    Some(Token::RParen) => {}
                    _ => return Err(FormulaError::UnmatchedParen),
                }

                self.call_function(&name, &args)
            }
            Some(Token::LParen) => {
                let val = self.parse_expression()?;
                match self.advance() {
                    Some(Token::RParen) => Ok(val),
                    _ => Err(FormulaError::UnmatchedParen),
                }
            }
            Some(other) => Err(FormulaError::UnexpectedToken(
                format!("{:?}", other),
                self.pos - 1,
            )),
            None => Err(FormulaError::UnexpectedEnd),
        }
    }

    fn call_function(&self, name: &str, args: &[f64]) -> Result<f64, FormulaError> {
        match name {
            "max" => {
                if args.len() < 2 {
                    return Err(FormulaError::InvalidArgCount("max".into(), 2, args.len()));
                }
                Ok(args.iter().copied().fold(f64::NEG_INFINITY, f64::max))
            }
            "min" => {
                if args.len() < 2 {
                    return Err(FormulaError::InvalidArgCount("min".into(), 2, args.len()));
                }
                Ok(args.iter().copied().fold(f64::INFINITY, f64::min))
            }
            "round" => {
                if args.len() != 1 {
                    return Err(FormulaError::InvalidArgCount(
                        "round".into(),
                        1,
                        args.len(),
                    ));
                }
                Ok(args[0].round())
            }
            "abs" => {
                if args.len() != 1 {
                    return Err(FormulaError::InvalidArgCount("abs".into(), 1, args.len()));
                }
                Ok(args[0].abs())
            }
            "sqrt" => {
                if args.len() != 1 {
                    return Err(FormulaError::InvalidArgCount(
                        "sqrt".into(),
                        1,
                        args.len(),
                    ));
                }
                Ok(args[0].sqrt())
            }
            _ => Err(FormulaError::UnknownFunction(name.into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> HashMap<String, f64> {
        let mut m = HashMap::new();
        m.insert("width".into(), 10.0);
        m.insert("height".into(), 5.0);
        m.insert("depth".into(), 2.0);
        m
    }

    #[test]
    fn test_basic_arithmetic() {
        let p = HashMap::new();
        assert!((evaluate_formula("2 + 3", &p).unwrap() - 5.0).abs() < 0.001);
        assert!((evaluate_formula("10 - 4", &p).unwrap() - 6.0).abs() < 0.001);
        assert!((evaluate_formula("3 * 4", &p).unwrap() - 12.0).abs() < 0.001);
        assert!((evaluate_formula("15 / 3", &p).unwrap() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_operator_precedence() {
        let p = HashMap::new();
        assert!((evaluate_formula("2 + 3 * 4", &p).unwrap() - 14.0).abs() < 0.001);
        assert!((evaluate_formula("(2 + 3) * 4", &p).unwrap() - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_variable_substitution() {
        let p = params();
        assert!((evaluate_formula("${width} * ${height}", &p).unwrap() - 50.0).abs() < 0.001);
        assert!(
            (evaluate_formula("${width} * ${height} * ${depth}", &p).unwrap() - 100.0).abs()
                < 0.001
        );
    }

    #[test]
    fn test_functions() {
        let p = HashMap::new();
        assert!((evaluate_formula("max(3, 7)", &p).unwrap() - 7.0).abs() < 0.001);
        assert!((evaluate_formula("min(3, 7)", &p).unwrap() - 3.0).abs() < 0.001);
        assert!((evaluate_formula("round(3.7)", &p).unwrap() - 4.0).abs() < 0.001);
        assert!((evaluate_formula("abs(-5)", &p).unwrap() - 5.0).abs() < 0.001);
        assert!((evaluate_formula("sqrt(16)", &p).unwrap() - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_nested_functions() {
        let p = HashMap::new();
        let result = evaluate_formula("max(sqrt(16), min(5, 3))", &p).unwrap();
        assert!((result - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_complex_formula() {
        let p = params();
        let result =
            evaluate_formula("(${width} * ${height} + ${depth}) * 1.15", &p).unwrap();
        assert!((result - 59.8).abs() < 0.001);
    }

    #[test]
    fn test_unary_minus() {
        let p = HashMap::new();
        assert!((evaluate_formula("-5 + 10", &p).unwrap() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_division_by_zero() {
        let p = HashMap::new();
        assert!(evaluate_formula("1 / 0", &p).is_err());
    }

    #[test]
    fn test_unknown_variable() {
        let p = HashMap::new();
        assert!(evaluate_formula("${unknown}", &p).is_err());
    }

    #[test]
    fn test_unknown_function() {
        let p = HashMap::new();
        assert!(evaluate_formula("foo(1)", &p).is_err());
    }
}
