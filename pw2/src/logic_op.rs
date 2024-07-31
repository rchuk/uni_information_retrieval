use anyhow::{anyhow, Result};
use std::str::Chars;

#[derive(Clone, Debug)]
enum Token {
    Term(String),
    And,
    Or,
    Not,
    LeftBracket,
    RightBracket
}

impl Token {
    pub fn precedence(&self) -> usize {
        match self {
            Token::Not => 3,
            Token::And => 2,
            Token::Or => 1,
            _ => 0,
        }
    }
}

struct Lexer<'a> {
    iter: Chars<'a>
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer { iter: input.chars() }
    }

    pub fn lex(mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut word = String::new();
        while let Some(ch) = self.iter.next() {
            if ch.is_alphabetic() || (ch.eq(&'\'') && !word.is_empty()) {
                ch.to_lowercase().for_each(|ch| word.push(ch));

                continue;
            }

            if !word.is_empty() {
                let mut new_word = String::new();
                std::mem::swap(&mut word, &mut new_word);

                tokens.push(Token::Term(new_word));
            }

            if ch.is_whitespace() {
                continue;
            }

            let operator = match ch {
                '&' => Token::And,
                '|' => Token::Or,
                '!' => Token::Not,
                '(' => Token::LeftBracket,
                ')' => Token::RightBracket,
                _ => return Err(anyhow!("Encountered invalid character: '{ch}'"))
            };

            tokens.push(operator);
        }

        if !word.is_empty() {
            tokens.push(Token::Term(word));
        }

        Ok(tokens)
    }
}

#[derive(Debug)]
pub enum LogicNode {
    False,
    Term(String),
    And(Box<LogicNode>, Box<LogicNode>),
    Or(Box<LogicNode>, Box<LogicNode>),
    Not(Box<LogicNode>)
}

struct Parser {
    tokens: Vec<Token>
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens }
    }

    pub fn parse(mut self) -> Result<LogicNode> {
        let mut operand_stack = Vec::new();
        let mut operator_stack = Vec::<Token>::new();

        let mut iter = self.tokens.into_iter();
        while let Some(token) = iter.next() {
            match token {
                Token::Term(term) => {
                    operand_stack.push(LogicNode::Term(term));
                },
                Token::And | Token::Or | Token::Not => {
                    while let Some(op) = operator_stack.last() {
                        if op.precedence() < token.precedence() {
                            break;
                        }

                        Self::construct_operator(&mut operator_stack, &mut operand_stack)?;
                    }

                    operator_stack.push(token);
                },
                Token::LeftBracket => {
                    operator_stack.push(token);
                },
                Token::RightBracket => {
                    while let Some(op) = operator_stack.last() {
                        if let Token::LeftBracket = op {
                            operator_stack.pop();
                            break;
                        }

                        Self::construct_operator(&mut operator_stack, &mut operand_stack)?;
                    }
                }
            }
        }

        while !operator_stack.is_empty() {
            Self::construct_operator(&mut operator_stack, &mut operand_stack)?;
        }

        Ok(operand_stack.pop().unwrap_or(LogicNode::False))
    }

    fn construct_operator(operator_stack: &mut Vec<Token>, operand_stack: &mut Vec<LogicNode>) -> Result<()> {
        let op = operator_stack.pop().ok_or(anyhow!("Expected operator"))?;
        Ok(match op {
            Token::And => {
                let (lhs, rhs) = Self::pop_binary_operand(operand_stack)?;
                operand_stack.push(LogicNode::And(Box::new(lhs), Box::new(rhs)));
            }
            Token::Or => {
                let (lhs, rhs) = Self::pop_binary_operand(operand_stack)?;
                operand_stack.push(LogicNode::Or(Box::new(lhs), Box::new(rhs)));
            }
            Token::Not => {
                let operand = Self::pop_unary_operand(operand_stack)?;
                operand_stack.push(LogicNode::Not(Box::new(operand)));
            }
            _ => return Err(anyhow!("Unexpected operator {op:?}"))
        })
    }

    fn pop_unary_operand(operand_stack: &mut Vec<LogicNode>) -> Result<LogicNode> {
        operand_stack.pop().ok_or(anyhow!("Missing argument"))
    }

    fn pop_binary_operand(operand_stack: &mut Vec<LogicNode>) -> Result<(LogicNode, LogicNode)> {
        Ok((
            Self::pop_unary_operand(operand_stack)?,
            Self::pop_unary_operand(operand_stack)?
        ))
    }
}

pub fn parse_logic_expr(input: &str) -> Result<LogicNode> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex()?;
    let mut parser = Parser::new(tokens);

    parser.parse()
}
