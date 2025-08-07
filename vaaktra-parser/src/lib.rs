//! Parser for the Sanskrit programming language
//! Converts tokens into an Abstract Syntax Tree (AST)

pub mod ast;
mod span;

use sanskrit_lexer::Token;
use std::iter::Peekable;
use thiserror::Error;

/// Represents a parsing error
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected}, found {found:?}")]
    UnexpectedToken {
        expected: String,
        found: Option<Token>,
        span: (usize, usize),
    },
    
    #[error("Unexpected end of input")]
    UnexpectedEof,
    
    #[error("Invalid number: {0}")]
    InvalidNumber(#[from] std::num::ParseIntError),
    
    #[error("Invalid syntax: {0}")]
    SyntaxError(String),
}

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, ParseError>;

/// The main parser struct
pub struct Parser<I>
where
    I: Iterator<Item = (Token, std::ops::Range<usize>)>,
{
    tokens: Peekable<I>,
    current_span: Option<std::ops::Range<usize>>,
}

impl<I> Parser<I>
where
    I: Iterator<Item = (Token, std::ops::Range<usize>)>,
{
    /// Create a new parser from an iterator of tokens
    pub fn new(tokens: I) -> Self {
        Parser {
            tokens: tokens.peekable(),
            current_span: None,
        }
    }
    
    /// Parse a complete program
    pub fn parse_program(&mut self) -> ParseResult<ast::Program> {
        let mut statements = Vec::new();
        
        while self.peek().is_some() {
            statements.push(self.parse_statement()?);
        }
        
        Ok(ast::Program { statements })
    }
    
    /// Parse a statement
    fn parse_statement(&mut self) -> ParseResult<ast::Statement> {
        match self.peek() {
            Some(Token::Main) => self.parse_function_decl(),
            Some(Token::Int) | Some(Token::Bool) | Some(Token::String) => {
                self.parse_variable_decl()
            }
            Some(Token::LBrace) => self.parse_block(),
            Some(Token::If) => self.parse_if_statement(),
            Some(Token::While) => self.parse_while_statement(),
            Some(Token::Return) => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }
    
    // Implementation of other parsing methods...
    // (parse_expression, parse_function_decl, parse_variable_decl, etc.)
    
    /// Get the next token
    fn next(&mut self) -> Option<Token> {
        if let Some((token, span)) = self.tokens.next() {
            self.current_span = Some(span);
            Some(token)
        } else {
            None
        }
    }
    
    /// Peek at the next token without consuming it
    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek().map(|(token, _)| token)
    }
    
    /// Expect a specific token
    fn expect(&mut self, expected: Token) -> ParseResult<()> {
        match self.next() {
            Some(token) if token == expected => Ok(()),
            found => {
                let span = self.current_span.clone().unwrap_or(0..0);
                Err(ParseError::UnexpectedToken {
                    expected: format!("{:?}", expected),
                    found,
                    span: (span.start, span.end),
                })
            }
        }
    }
    
    /// Check if the next token matches the expected token
    fn matches(&mut self, expected: Token) -> bool {
        if let Some(token) = self.peek() {
            if *token == expected {
                self.next();
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sanskrit_lexer::Lexer;
    
    #[test]
    fn test_parse_empty_program() {
        let input = "";
        let tokens = Lexer::new(input);
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        assert!(program.statements.is_empty());
    }
    
    // More tests will be added as we implement more parsing functionality
}
