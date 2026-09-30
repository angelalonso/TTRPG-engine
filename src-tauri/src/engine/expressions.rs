use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpressionLimits {
    pub max_nodes: usize,
    pub max_depth: usize,
}

impl Default for ExpressionLimits {
    fn default() -> Self {
        Self {
            max_nodes: 256,
            max_depth: 32,
        }
    }
}

pub trait FactProvider {
    fn fact(&self, name: &str) -> Option<f64>;

    fn call(&self, name: &str, arguments: &[String]) -> Option<f64> {
        let key = format!("{name}({})", arguments.join(","));
        self.fact(&key)
    }
}

impl FactProvider for HashMap<String, f64> {
    fn fact(&self, name: &str) -> Option<f64> {
        self.get(name).copied()
    }
}

impl FactProvider for HashMap<&str, f64> {
    fn fact(&self, name: &str) -> Option<f64> {
        self.get(name).copied()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(f64),
    StringLiteral(String),
    Fact(String),
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Function {
        name: FunctionName,
        arguments: Vec<Expression>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Plus,
    Minus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionName {
    Min,
    Max,
    Clamp,
    Characteristic,
    InventoryCount,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionError {
    Empty {
        position: usize,
    },
    UnexpectedCharacter {
        position: usize,
        character: char,
    },
    UnexpectedToken {
        position: usize,
        expected: String,
    },
    InvalidNumber {
        position: usize,
        source: String,
    },
    UnknownFunction {
        position: usize,
        name: String,
    },
    WrongArgumentCount {
        position: usize,
        name: String,
        expected: usize,
        actual: usize,
    },
    UnknownFact {
        position: usize,
        name: String,
    },
    DivisionByZero {
        position: usize,
    },
    NonFiniteResult {
        position: usize,
    },
    ComplexityLimitExceeded {
        position: usize,
        limit: usize,
    },
    DepthLimitExceeded {
        position: usize,
        limit: usize,
    },
}

impl ExpressionError {
    pub fn position(&self) -> usize {
        match self {
            Self::Empty { position }
            | Self::UnexpectedCharacter { position, .. }
            | Self::UnexpectedToken { position, .. }
            | Self::InvalidNumber { position, .. }
            | Self::UnknownFunction { position, .. }
            | Self::WrongArgumentCount { position, .. }
            | Self::UnknownFact { position, .. }
            | Self::DivisionByZero { position }
            | Self::NonFiniteResult { position }
            | Self::ComplexityLimitExceeded { position, .. }
            | Self::DepthLimitExceeded { position, .. } => *position,
        }
    }
}

impl fmt::Display for ExpressionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "expression error at position {}: ",
            self.position()
        )?;
        match self {
            Self::Empty { .. } => write!(formatter, "expression is empty"),
            Self::UnexpectedCharacter { character, .. } => {
                write!(formatter, "unexpected character '{character}'")
            }
            Self::UnexpectedToken { expected, .. } => write!(formatter, "expected {expected}"),
            Self::InvalidNumber { source, .. } => write!(formatter, "invalid number '{source}'"),
            Self::UnknownFunction { name, .. } => write!(formatter, "unknown function '{name}'"),
            Self::WrongArgumentCount {
                name,
                expected,
                actual,
                ..
            } => write!(
                formatter,
                "function '{name}' expects {expected} arguments, got {actual}"
            ),
            Self::UnknownFact { name, .. } => write!(formatter, "unknown fact '{name}'"),
            Self::DivisionByZero { .. } => write!(formatter, "division by zero"),
            Self::NonFiniteResult { .. } => write!(formatter, "result is not finite"),
            Self::ComplexityLimitExceeded { limit, .. } => {
                write!(formatter, "complexity limit of {limit} nodes exceeded")
            }
            Self::DepthLimitExceeded { limit, .. } => {
                write!(formatter, "depth limit of {limit} exceeded")
            }
        }
    }
}

impl std::error::Error for ExpressionError {}

impl Expression {
    pub fn parse(source: &str) -> Result<Self, ExpressionError> {
        Self::parse_with_limits(source, ExpressionLimits::default())
    }

    pub fn parse_with_limits(
        source: &str,
        limits: ExpressionLimits,
    ) -> Result<Self, ExpressionError> {
        let mut parser = Parser::new(source, limits);
        let expression = parser.parse_expression()?;
        if let Some(token) = parser.peek() {
            return Err(ExpressionError::UnexpectedToken {
                position: token.position,
                expected: "end of expression".to_string(),
            });
        }
        Ok(expression)
    }

    pub fn evaluate<P: FactProvider>(&self, facts: &P) -> Result<f64, ExpressionError> {
        self.evaluate_at(facts, 0)
    }

    fn evaluate_at<P: FactProvider>(
        &self,
        facts: &P,
        position: usize,
    ) -> Result<f64, ExpressionError> {
        let value = match self {
            Self::Literal(value) => *value,
            Self::StringLiteral(name) => {
                return Err(ExpressionError::UnknownFact {
                    position,
                    name: name.clone(),
                })
            }
            Self::Fact(name) => facts
                .fact(name)
                .ok_or_else(|| ExpressionError::UnknownFact {
                    position,
                    name: name.clone(),
                })?,
            Self::Unary { operator, operand } => {
                let value = operand.evaluate_at(facts, position)?;
                match operator {
                    UnaryOperator::Plus => value,
                    UnaryOperator::Minus => -value,
                }
            }
            Self::Binary {
                operator,
                left,
                right,
            } => {
                let left = left.evaluate_at(facts, position)?;
                let right = right.evaluate_at(facts, position)?;
                match operator {
                    BinaryOperator::Add => left + right,
                    BinaryOperator::Subtract => left - right,
                    BinaryOperator::Multiply => left * right,
                    BinaryOperator::Divide if right == 0.0 => {
                        return Err(ExpressionError::DivisionByZero { position });
                    }
                    BinaryOperator::Divide => left / right,
                }
            }
            Self::Function {
                name: FunctionName::Characteristic,
                arguments,
            } => {
                let argument = string_argument(arguments, 0, position)?;
                facts.call("characteristic", &[argument]).ok_or_else(|| {
                    ExpressionError::UnknownFact {
                        position,
                        name: "characteristic".to_string(),
                    }
                })?
            }
            Self::Function {
                name: FunctionName::InventoryCount,
                arguments,
            } => {
                let definition = string_argument(arguments, 0, position)?;
                let value = string_argument(arguments, 1, position)?;
                facts
                    .call("inventory_count", &[definition, value])
                    .ok_or_else(|| ExpressionError::UnknownFact {
                        position,
                        name: "inventory_count".to_string(),
                    })?
            }
            Self::Function { name, arguments } => {
                let values = arguments
                    .iter()
                    .map(|argument| argument.evaluate_at(facts, position))
                    .collect::<Result<Vec<_>, _>>()?;
                match name {
                    FunctionName::Min => values.into_iter().fold(f64::INFINITY, f64::min),
                    FunctionName::Max => values.into_iter().fold(f64::NEG_INFINITY, f64::max),
                    FunctionName::Clamp => values[0].clamp(values[1], values[2]),
                    FunctionName::Characteristic | FunctionName::InventoryCount => {
                        unreachable!("fact functions are evaluated before numeric functions")
                    }
                }
            }
        };
        if value.is_finite() {
            Ok(value)
        } else {
            Err(ExpressionError::NonFiniteResult { position })
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Token {
    kind: TokenKind,
    position: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Number(f64),
    String(String),
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,
    RightParen,
    Comma,
}

struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    token_error: Option<ExpressionError>,
    index: usize,
    limits: ExpressionLimits,
    nodes: usize,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str, limits: ExpressionLimits) -> Self {
        let (tokens, token_error) = match tokenize(source) {
            Ok(tokens) => (tokens, None),
            Err(error) => (Vec::new(), Some(error)),
        };
        Self {
            source,
            tokens,
            token_error,
            index: 0,
            limits,
            nodes: 0,
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, ExpressionError> {
        if let Some(error) = self.token_error.take() {
            return Err(error);
        }
        if self.source.trim().is_empty() {
            return Err(ExpressionError::Empty { position: 0 });
        }
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<Expression, ExpressionError> {
        let mut expression = self.parse_multiplicative()?;
        while let Some(token) = self.peek().cloned() {
            let operator = match token.kind {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Subtract,
                _ => break,
            };
            self.next();
            let right = self.parse_multiplicative()?;
            expression = self.node(
                Expression::Binary {
                    operator,
                    left: Box::new(expression),
                    right: Box::new(right),
                },
                token.position,
            )?;
        }
        Ok(expression)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, ExpressionError> {
        let mut expression = self.parse_unary()?;
        while let Some(token) = self.peek().cloned() {
            let operator = match token.kind {
                TokenKind::Star => BinaryOperator::Multiply,
                TokenKind::Slash => BinaryOperator::Divide,
                _ => break,
            };
            self.next();
            let right = self.parse_unary()?;
            expression = self.node(
                Expression::Binary {
                    operator,
                    left: Box::new(expression),
                    right: Box::new(right),
                },
                token.position,
            )?;
        }
        Ok(expression)
    }

    fn parse_unary(&mut self) -> Result<Expression, ExpressionError> {
        if let Some(token) = self.peek().cloned() {
            let operator = match token.kind {
                TokenKind::Plus => Some(UnaryOperator::Plus),
                TokenKind::Minus => Some(UnaryOperator::Minus),
                _ => None,
            };
            if let Some(operator) = operator {
                self.next();
                let operand = self.parse_unary()?;
                return self.node(
                    Expression::Unary {
                        operator,
                        operand: Box::new(operand),
                    },
                    token.position,
                );
            }
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression, ExpressionError> {
        let token = self
            .next()
            .ok_or_else(|| ExpressionError::UnexpectedToken {
                position: self.source.len(),
                expected: "number, fact, or '('".to_string(),
            })?;
        match token.kind {
            TokenKind::Number(value) => self.node(Expression::Literal(value), token.position),
            TokenKind::Identifier(name) => {
                if !matches!(
                    self.peek().map(|token| &token.kind),
                    Some(TokenKind::LeftParen)
                ) {
                    return self.node(Expression::Fact(name), token.position);
                }
                self.next();
                let function_name = match name.as_str() {
                    "min" => FunctionName::Min,
                    "max" => FunctionName::Max,
                    "clamp" => FunctionName::Clamp,
                    "characteristic" => FunctionName::Characteristic,
                    "inventory_count" => FunctionName::InventoryCount,
                    _ => {
                        return Err(ExpressionError::UnknownFunction {
                            position: token.position,
                            name,
                        })
                    }
                };
                let mut arguments = Vec::new();
                if !matches!(
                    self.peek().map(|token| &token.kind),
                    Some(TokenKind::RightParen)
                ) {
                    loop {
                        arguments.push(self.parse_additive()?);
                        if !matches!(self.peek().map(|token| &token.kind), Some(TokenKind::Comma)) {
                            break;
                        }
                        self.next();
                    }
                }
                let closing = self.expect(TokenKind::RightParen, "')'")?;
                let expected = match function_name {
                    FunctionName::Min | FunctionName::Max => 2,
                    FunctionName::Clamp => 3,
                    FunctionName::Characteristic => 1,
                    FunctionName::InventoryCount => 2,
                };
                let valid_count = match function_name {
                    FunctionName::Min | FunctionName::Max => arguments.len() >= expected,
                    FunctionName::Clamp
                    | FunctionName::Characteristic
                    | FunctionName::InventoryCount => arguments.len() == expected,
                };
                if !valid_count {
                    return Err(ExpressionError::WrongArgumentCount {
                        position: closing.position,
                        name: name_for_function(function_name).to_string(),
                        expected,
                        actual: arguments.len(),
                    });
                }
                self.node(
                    Expression::Function {
                        name: function_name,
                        arguments,
                    },
                    token.position,
                )
            }
            TokenKind::String(value) => self.node(Expression::StringLiteral(value), token.position),
            TokenKind::LeftParen => {
                let expression = self.parse_additive()?;
                self.expect(TokenKind::RightParen, "')'")?;
                Ok(expression)
            }
            _ => Err(ExpressionError::UnexpectedToken {
                position: token.position,
                expected: "number, fact, or '('".to_string(),
            }),
        }
    }

    fn node(
        &mut self,
        expression: Expression,
        position: usize,
    ) -> Result<Expression, ExpressionError> {
        self.nodes += 1;
        if self.nodes > self.limits.max_nodes {
            return Err(ExpressionError::ComplexityLimitExceeded {
                position,
                limit: self.limits.max_nodes,
            });
        }
        if expression.depth() > self.limits.max_depth {
            return Err(ExpressionError::DepthLimitExceeded {
                position,
                limit: self.limits.max_depth,
            });
        }
        Ok(expression)
    }

    fn expect(&mut self, expected: TokenKind, description: &str) -> Result<Token, ExpressionError> {
        let token = self
            .next()
            .ok_or_else(|| ExpressionError::UnexpectedToken {
                position: self.source.len(),
                expected: description.to_string(),
            })?;
        if std::mem::discriminant(&token.kind) != std::mem::discriminant(&expected) {
            return Err(ExpressionError::UnexpectedToken {
                position: token.position,
                expected: description.to_string(),
            });
        }
        Ok(token)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.index).cloned();
        self.index += usize::from(token.is_some());
        token
    }
}

impl Expression {
    fn depth(&self) -> usize {
        match self {
            Self::Literal(_) | Self::StringLiteral(_) | Self::Fact(_) => 1,
            Self::Unary { operand, .. } => 1 + operand.depth(),
            Self::Binary { left, right, .. } => 1 + left.depth().max(right.depth()),
            Self::Function { arguments, .. } => {
                1 + arguments.iter().map(Self::depth).max().unwrap_or(0)
            }
        }
    }
}

fn name_for_function(function: FunctionName) -> &'static str {
    match function {
        FunctionName::Min => "min",
        FunctionName::Max => "max",
        FunctionName::Clamp => "clamp",
        FunctionName::Characteristic => "characteristic",
        FunctionName::InventoryCount => "inventory_count",
    }
}

fn string_argument(
    arguments: &[Expression],
    index: usize,
    position: usize,
) -> Result<String, ExpressionError> {
    match arguments.get(index) {
        Some(Expression::StringLiteral(value)) => Ok(value.clone()),
        Some(Expression::Fact(value)) => Ok(value.clone()),
        _ => Err(ExpressionError::UnexpectedToken {
            position,
            expected: "a quoted string".to_string(),
        }),
    }
}

fn tokenize(source: &str) -> Result<Vec<Token>, ExpressionError> {
    let mut tokens = Vec::new();
    let bytes = source.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let character = source[index..].chars().next().unwrap();
        if character.is_ascii_whitespace() {
            index += character.len_utf8();
            continue;
        }
        let token = match character {
            '+' => (TokenKind::Plus, 1),
            '-' => (TokenKind::Minus, 1),
            '*' => (TokenKind::Star, 1),
            '/' => (TokenKind::Slash, 1),
            '(' => (TokenKind::LeftParen, 1),
            ')' => (TokenKind::RightParen, 1),
            ',' => (TokenKind::Comma, 1),
            '"' | '\'' => {
                let quote = character;
                let start = index;
                index += quote.len_utf8();
                let mut value = String::new();
                let mut closed = false;
                while index < bytes.len() {
                    let next = source[index..].chars().next().unwrap();
                    index += next.len_utf8();
                    if next == quote {
                        closed = true;
                        break;
                    }
                    if next == '\\' && index < bytes.len() {
                        let escaped = source[index..].chars().next().unwrap();
                        index += escaped.len_utf8();
                        value.push(escaped);
                    } else {
                        value.push(next);
                    }
                }
                if !closed {
                    return Err(ExpressionError::UnexpectedToken {
                        position: start,
                        expected: "closing quote".to_string(),
                    });
                }
                tokens.push(Token {
                    kind: TokenKind::String(value),
                    position: start,
                });
                continue;
            }
            '0'..='9' | '.' => {
                let start = index;
                let mut end = index;
                while end < bytes.len()
                    && (bytes[end].is_ascii_digit()
                        || matches!(bytes[end], b'.' | b'e' | b'E' | b'+' | b'-'))
                {
                    if end > start
                        && matches!(bytes[end], b'+' | b'-')
                        && !matches!(bytes[end - 1], b'e' | b'E')
                    {
                        break;
                    }
                    end += 1;
                }
                let source_number = &source[start..end];
                let value =
                    source_number
                        .parse::<f64>()
                        .map_err(|_| ExpressionError::InvalidNumber {
                            position: start,
                            source: source_number.to_string(),
                        })?;
                if !value.is_finite() {
                    return Err(ExpressionError::InvalidNumber {
                        position: start,
                        source: source_number.to_string(),
                    });
                }
                tokens.push(Token {
                    kind: TokenKind::Number(value),
                    position: start,
                });
                index = end;
                continue;
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let start = index;
                let mut end = index + character.len_utf8();
                while end < bytes.len()
                    && (bytes[end].is_ascii_alphanumeric()
                        || matches!(bytes[end], b'_' | b'.' | b':' | b'-'))
                {
                    end += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::Identifier(source[start..end].to_string()),
                    position: start,
                });
                index = end;
                continue;
            }
            _ => {
                return Err(ExpressionError::UnexpectedCharacter {
                    position: index,
                    character,
                })
            }
        };
        tokens.push(Token {
            kind: token.0,
            position: index,
        });
        index += token.1;
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(count: f64) -> HashMap<String, f64> {
        HashMap::from([("count".to_string(), count)])
    }

    #[test]
    fn probability_example_is_bounded_and_uses_fact_counts() {
        let expression = Expression::parse("clamp(0.30 + count * 0.05, 0.30, 0.90)").unwrap();
        for (count, expected) in [(0.0, 0.30), (3.0, 0.45), (20.0, 0.90)] {
            assert_eq!(expression.evaluate(&facts(count)).unwrap(), expected);
        }
    }

    #[test]
    fn typed_inventory_and_characteristic_functions_are_supported() {
        let expression = Expression::parse(
            r#"clamp(0.30 + 0.05 * inventory_count("type", "ingredient"), 0, 0.90)"#,
        )
        .unwrap();
        let facts = HashMap::from([
            ("inventory_count(type,ingredient)".to_string(), 3.0),
            ("characteristic(skill)".to_string(), 5.0),
        ]);
        assert_eq!(expression.evaluate(&facts), Ok(0.45));
        assert_eq!(
            Expression::parse(r#"characteristic("skill")"#)
                .unwrap()
                .evaluate(&facts),
            Ok(5.0)
        );
    }

    #[test]
    fn precedence_negatives_and_functions_work() {
        let facts = HashMap::from([("x".to_string(), 4.0)]);
        assert_eq!(
            Expression::parse("-2 + 3 * x").unwrap().evaluate(&facts),
            Ok(10.0)
        );
        assert_eq!(
            Expression::parse("max(1, min(8, 3 + 2))")
                .unwrap()
                .evaluate(&facts),
            Ok(5.0)
        );
    }

    #[test]
    fn diagnostics_and_safety_errors_include_positions() {
        assert_eq!(
            Expression::parse("1 / 0")
                .unwrap()
                .evaluate(&HashMap::<String, f64>::new()),
            Err(ExpressionError::DivisionByZero { position: 0 })
        );
        assert!(matches!(
            Expression::parse("evil(1)"),
            Err(ExpressionError::UnknownFunction { position: 0, .. })
        ));
        assert!(matches!(
            Expression::parse("missing")
                .unwrap()
                .evaluate(&HashMap::<String, f64>::new()),
            Err(ExpressionError::UnknownFact { position: 0, .. })
        ));
        assert!(matches!(
            Expression::parse("1; std::process::exit(1)"),
            Err(ExpressionError::UnexpectedCharacter { position: 1, .. })
        ));
    }

    #[test]
    fn complexity_is_bounded() {
        let limits = ExpressionLimits {
            max_nodes: 3,
            max_depth: 32,
        };
        assert!(matches!(
            Expression::parse_with_limits("1 + 2 + 3", limits),
            Err(ExpressionError::ComplexityLimitExceeded { .. })
        ));
    }
}
