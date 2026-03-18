#![warn(clippy::pedantic)]
// Evaluation of easy arithmetical expressions over usize integers.
//
// Supports: non-negative integers, +, -, *, /, parentheses.
// Operator precedence: * and / bind tighter than + and -.
// Subtraction underflow (wrapping below zero) is an error.
//
// Grammar (recursive descent):
//
//   expr   = term   (('+' | '-') term)*
//   term   = factor (('*' | '/') factor)*
//   factor = '(' expr ')' | number
//   number = [0-9]+
//
// Usage:
//   let result = eval("3 + 4 * 2");   // Ok(11)
//   let result = eval("(1 + 2) * 3"); // Ok(9)

/// # Errors
///
/// Returns `Err` if the expression contains unknown characters, mismatched
/// parentheses, division by zero, or subtraction that would underflow.
pub fn eval(input: &str) -> Result<usize, String> {
    let tokens = tokenize(input)?;
    let mut p = Parser { tokens, pos: 0 };
    let value = p.expr()?;
    if p.pos != p.tokens.len() {
        return Err(format!("unexpected token at position {}", p.pos));
    }
    Ok(value)
}

// ── Tokeniser ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(usize),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' => { chars.next(); }
            '0'..='9' => {
                let mut s = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() { s.push(d); chars.next(); } else { break; }
                }
                let n: usize = s.parse().map_err(|_| format!("number too large: {s}"))?;
                tokens.push(Token::Num(n));
            }
            '+' => { tokens.push(Token::Plus);   chars.next(); }
            '-' => { tokens.push(Token::Minus);  chars.next(); }
            '*' => { tokens.push(Token::Star);   chars.next(); }
            '/' => { tokens.push(Token::Slash);  chars.next(); }
            '(' => { tokens.push(Token::LParen); chars.next(); }
            ')' => { tokens.push(Token::RParen); chars.next(); }
            other => return Err(format!("unknown character: '{other}'")),
        }
    }
    Ok(tokens)
}

// ── Parser ───────────────────────────────────────────────────────────────────

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn consume(&mut self) {
        self.pos += 1;
    }

    // expr = term (('+' | '-') term)*
    fn expr(&mut self) -> Result<usize, String> {
        let mut val = self.term()?;
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.consume();
                    val += self.term()?;
                }
                Some(Token::Minus) => {
                    self.consume();
                    let rhs = self.term()?;
                    val = val.checked_sub(rhs)
                        .ok_or_else(|| format!("subtraction underflow: {val} - {rhs}"))?;
                }
                _ => break,
            }
        }
        Ok(val)
    }

    // term = factor (('*' | '/') factor)*
    fn term(&mut self) -> Result<usize, String> {
        let mut val = self.factor()?;
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.consume();
                    val *= self.factor()?;
                }
                Some(Token::Slash) => {
                    self.consume();
                    let rhs = self.factor()?;
                    if rhs == 0 {
                        return Err("division by zero".into());
                    }
                    val /= rhs;
                }
                _ => break,
            }
        }
        Ok(val)
    }

    // factor = '(' expr ')' | number
    fn factor(&mut self) -> Result<usize, String> {
        match self.peek().cloned() {
            Some(Token::LParen) => {
                self.consume();
                let val = self.expr()?;
                if self.peek() != Some(&Token::RParen) {
                    return Err("expected ')'".into());
                }
                self.consume();
                Ok(val)
            }
            Some(Token::Num(n)) => {
                self.consume();
                Ok(n)
            }
            other => Err(format!("expected number or '(', got {other:?}")),
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::eval;

    #[test]
    fn test_number()        { assert_eq!(eval("42"),          Ok(42)); }
    #[test]
    fn test_add()           { assert_eq!(eval("1 + 2"),        Ok(3)); }
    #[test]
    fn test_sub()           { assert_eq!(eval("10 - 3"),       Ok(7)); }
    #[test]
    fn test_mul()           { assert_eq!(eval("4 * 5"),        Ok(20)); }
    #[test]
    fn test_div()           { assert_eq!(eval("20 / 4"),       Ok(5)); }
    #[test]
    fn test_precedence()    { assert_eq!(eval("3 + 4 * 2"),   Ok(11)); }
    #[test]
    fn test_parens()        { assert_eq!(eval("(3 + 4) * 2"), Ok(14)); }
    #[test]
    fn test_nested_parens() { assert_eq!(eval("(1+(2*3))"),    Ok(7)); }
    #[test]
    fn test_underflow()     { assert!(eval("3 - 5").is_err()); }
    #[test]
    fn test_div_by_zero()   { assert!(eval("1 / 0").is_err()); }
    #[test]
    fn test_unknown_char()  { assert!(eval("1 $ 2").is_err()); }
}
