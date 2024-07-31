use std::iter::Peekable;
use anyhow::{anyhow, Context, Result};
use std::str::{Chars, FromStr};

#[derive(Eq, PartialEq, Clone, Debug)]
enum Token {
    Term(String),
    Number(usize),
    Ampersand,
    Pipe,
    Exclaim,
    LeftRoundBracket,
    RightRoundBracket,
    LeftCurlyBracket,
    RightCurlyBracket,
    GreaterThan,
    DoubleQuotes,
    Backslash
}

struct Lexer<'a> {
    iter: Peekable<Chars<'a>>
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer { iter: input.chars().peekable() }
    }

    pub fn lex(mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        while let Some(&ch) = self.iter.peek() {
            if let Some(term) = Self::try_consume_term(&mut self.iter) {
                tokens.push(term);
            } else if ch.is_whitespace() {
                Self::skip_whitespaces(&mut self.iter);
            } else if ch.is_ascii_digit() {
                self.iter.next();
                tokens.push(Self::consume_number_with_head(ch.to_string(), &mut self.iter)?);
            } else if let Some(punctuator) = Self::try_consume_punctuator(&mut self.iter) {
                tokens.push(punctuator);
            } else {
                return Err(anyhow!("Encountered invalid character: '{ch}'"))
            }
        }

        Ok(tokens)
    }

    fn try_consume_term(iter: &mut Peekable<impl Iterator<Item = char>>) -> Option<Token> {
        let mut word = String::new();
        while let Some(ch) = iter.peek() {
            if ch.is_alphabetic() || (ch.eq(&'\'') && !word.is_empty()) {
                ch.to_lowercase().for_each(|ch| word.push(ch));
                iter.next();
            } else if !word.is_empty() {
                return Some(Token::Term(word))
            } else {
                return None
            }
        }

        None
    }

    fn try_consume_punctuator(iter: &mut Peekable<impl Iterator<Item = char>>) -> Option<Token> {
        if let Some(ch) = iter.peek() {
            let punctuator = Some(match ch {
                '&' => Token::Ampersand,
                '|' => Token::Pipe,
                '!' => Token::Exclaim,
                '(' => Token::LeftRoundBracket,
                ')' => Token::RightRoundBracket,
                '{' => Token::LeftCurlyBracket,
                '}' => Token::RightCurlyBracket,
                '>' => Token::GreaterThan,
                '"' => Token::DoubleQuotes,
                '\\' => Token::Backslash,
                _ => return None
            });

            if punctuator.is_some() {
                iter.next();
            }

            punctuator
        } else {
            None
        }
    }

    fn consume_number_with_head(mut head: String, iter: &mut Peekable<impl Iterator<Item = char>>) -> Result<Token> {
        while let Some(&ch) = iter.peek() {
            if !ch.is_ascii_digit() {
                break;
            }

            head.push(ch);
            iter.next();
        }

        let number = usize::from_str(&head).context(anyhow!("Invalid number {head}"))?;
        Ok(Token::Number(number))
    }

    fn skip_whitespaces(iter: &mut Peekable<impl Iterator<Item = char>>) {
        while let Some(ch) = iter.peek() {
            if ch.is_whitespace() {
                iter.next();
            } else {
                break;
            }
        }
    }
}

#[derive(Clone, Debug)]
enum Operator {
    And,
    Or,
    Not,
    Near(usize),
    Next,
    LeftBracket,
    Subtract
}

impl Operator {
    pub fn precedence(&self) -> usize {
        match self {
            Operator::Next => 100,
            Operator::Near(_) => 50,
            Operator::Not => 4,
            Operator::Subtract => 3,
            Operator::And => 2,
            Operator::Or => 1,
            _ => 0,
        }
    }

    pub fn from_token(token: &Token) -> Option<Self> {
        Some(match token {
            Token::Ampersand => Operator::And,
            Token::Pipe => Operator::Or,
            Token::Exclaim => Operator::Not,
            Token::Backslash => Operator::Subtract,
            _ => return None
        })
    }
}


#[derive(Debug)]
pub enum LogicNode {
    False,
    Term(String),
    And(Box<LogicNode>, Box<LogicNode>),
    Or(Box<LogicNode>, Box<LogicNode>),
    Not(Box<LogicNode>),
    Near(Box<LogicNode>, Box<LogicNode>, usize, usize),
    Subtract(Box<LogicNode>, Box<LogicNode>)
}

struct Parser {
    tokens: Vec<Token>
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens }
    }

    pub fn parse(self) -> Result<LogicNode> {
        let mut operand_stack = Vec::new();
        let mut operator_stack = Vec::<Operator>::new();

        let mut iter = self.tokens.into_iter().peekable();
        while let Some(token) = iter.next() {
            match token {
                Token::Term(term) => {
                    operand_stack.push(LogicNode::Term(term));
                },
                Token::Ampersand | Token::Pipe | Token::Exclaim | Token::Backslash => {
                    let operator = Operator::from_token(&token)
                        .context(anyhow!("Programming error. Token {token:?} is not an operator."))?;

                    while let Some(op) = operator_stack.last() {
                        if op.precedence() < operator.precedence() {
                            break;
                        }

                        Self::construct_operator(&mut operator_stack, &mut operand_stack)?;
                    }

                    operator_stack.push(operator);
                },
                Token::LeftRoundBracket => {
                    operator_stack.push(Operator::LeftBracket);
                },
                Token::RightRoundBracket => {
                    while let Some(op) = operator_stack.last() {
                        if let Operator::LeftBracket = op {
                            operator_stack.pop();
                            break;
                        }

                        Self::construct_operator(&mut operator_stack, &mut operand_stack)?;
                    }
                },
                Token::LeftCurlyBracket => {
                    if let Some(Token::Number(distance)) = iter.next() {
                        if let Some(Token::RightCurlyBracket) = iter.next() {
                            operator_stack.push(Operator::Near(distance));
                        } else {
                            return Err(anyhow!("Expected closing '}}' bracket for 'near' operator"));
                        }
                    } else {
                        return Err(anyhow!("Expected number for 'near' operator"));
                    }
                },
                Token::GreaterThan => {
                    operator_stack.push(Operator::Next);
                },
                Token::DoubleQuotes => {
                    while let Some(token) = iter.peek() {
                        match token {
                            Token::Term(term) => {
                                operand_stack.push(LogicNode::Term(term.clone()));
                                iter.next();
                                if let Some(Token::Term(_)) = iter.peek() {
                                    operator_stack.push(Operator::Next);
                                }
                            },
                            Token::DoubleQuotes => break,
                            _ => return Err(anyhow!("Unexpected token {:?} inside phrase literal", token))
                        }
                    }
                    match iter.next() {
                        Some(Token::DoubleQuotes) => (),
                        _ => return Err(anyhow!("Unclosed phrase literal double quotes '\"'"))
                    };
                }
                _ => {
                    return Err(anyhow!("Unexpected token: {:?}", token));
                }
            }
        }

        while !operator_stack.is_empty() {
            Self::construct_operator(&mut operator_stack, &mut operand_stack)?;
        }

        if operand_stack.len() > 1 {
            return Err(anyhow!("Expected single expression"));
        }

        Ok(operand_stack.pop().unwrap_or(LogicNode::False))
    }

    fn construct_operator(operator_stack: &mut Vec<Operator>, operand_stack: &mut Vec<LogicNode>) -> Result<()> {
        let op = operator_stack.pop().ok_or(anyhow!("Expected operator"))?;
        Ok(match op {
            Operator::And => {
                let (lhs, rhs) = Self::pop_binary_operand(operand_stack)?;
                operand_stack.push(LogicNode::And(Box::new(lhs), Box::new(rhs)));
            }
            Operator::Or => {
                let (lhs, rhs) = Self::pop_binary_operand(operand_stack)?;
                operand_stack.push(LogicNode::Or(Box::new(lhs), Box::new(rhs)));
            }
            Operator::Not => {
                let operand = Self::pop_unary_operand(operand_stack)?;
                operand_stack.push(LogicNode::Not(Box::new(operand)));
            },
            Operator::Near(distance) => {
                let (lhs, rhs) = Self::pop_binary_operand(operand_stack)?;
                operand_stack.push(LogicNode::Near(Box::new(lhs), Box::new(rhs), distance, distance));
            },
            Operator::Next => {
                let (lhs, rhs) = Self::pop_binary_operand(operand_stack)?;
                operand_stack.push(LogicNode::Near(Box::new(lhs), Box::new(rhs), 0, 1));
            },
            Operator::Subtract => {
                let (lhs, rhs) = Self::pop_binary_operand(operand_stack)?;
                operand_stack.push(LogicNode::Subtract(Box::new(lhs), Box::new(rhs)));
            }
            _ => return Err(anyhow!("Unexpected operator {op:?}"))
        })
    }

    fn pop_unary_operand(operand_stack: &mut Vec<LogicNode>) -> Result<LogicNode> {
        operand_stack.pop().ok_or(anyhow!("Missing argument"))
    }

    fn pop_binary_operand(operand_stack: &mut Vec<LogicNode>) -> Result<(LogicNode, LogicNode)> {
        let (second, first) = (
            Self::pop_unary_operand(operand_stack)?,
            Self::pop_unary_operand(operand_stack)?
        );

        Ok((first, second))
    }
}

pub fn parse_logic_expr(input: &str) -> Result<LogicNode> {
    let lexer = Lexer::new(input);
    let tokens = lexer.lex()?;
    let parser = Parser::new(tokens);

    parser.parse()
}
