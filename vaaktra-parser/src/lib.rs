//! Parser for the Sanskrit programming language
//! Converts tokens into an Abstract Syntax Tree (AST)

pub mod ast;
mod span;

use vaaktra_lexer::Token;
use std::iter::Peekable;
use thiserror::Error;

use crate::ast::Span;

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
        
        Ok(ast::Program { items: statements, span: Span::dummy() })
    }
    
    /// Parse a statement
    fn parse_statement(&mut self) -> ParseResult<ast::Statement> {
        let start_pos = self.current_span().map(|s| s.start).unwrap_or(0);
        
        let stmt = match self.peek() {
            // Vedic-inspired keywords
            Some(Token::Dharma) => {
                // Dharma declarations are top-level items, wrap in an Item statement
                if let ast::Statement::Item(ast::Item::Dharma(dharma)) = self.parse_dharma_decl()? {
                    ast::Statement::Item(ast::Item::Dharma(dharma))
                } else {
                    return Err(ParseError::SyntaxError("Expected dharma declaration".to_string()));
                }
            },
            Some(Token::Mantra) => {
                // Mantra declarations are top-level items, wrap in an Item statement
                if let ast::Statement::Item(ast::Item::Mantra(mantra)) = self.parse_mantra_decl()? {
                    ast::Statement::Item(ast::Item::Mantra(mantra))
                } else {
                    return Err(ParseError::SyntaxError("Expected mantra declaration".to_string()));
                }
            },
            Some(Token::Sutra) => {
                // Sutra declarations are statements
                self.parse_sutra_decl()?
            },
            Some(Token::Yantra) => {
                // Yantra declarations are top-level items, wrap in an Item statement
                if let ast::Statement::Item(ast::Item::Yantra(yantra)) = self.parse_yantra_decl()? {
                    ast::Statement::Item(ast::Item::Yantra(yantra))
                } else {
                    return Err(ParseError::SyntaxError("Expected yantra declaration".to_string()));
                }
            },
            
            // Control flow
            Some(Token::If) => self.parse_if_statement()?,
            Some(Token::While) => self.parse_while_statement()?,
            Some(Token::Return) => self.parse_return_statement()?,
            
            // Blocks and expressions
            Some(Token::LBrace) => self.parse_block()?,
            _ => self.parse_expression_statement()?,
        };
        
        let end_pos = self.current_span().map(|s| s.end).unwrap_or(0);
        
        // Add span information to the statement if it doesn't have one
        match stmt {
            ast::Statement::Sutra(mut sutra) => {
                sutra.span = ast::Span::new(start_pos, end_pos, 0);
                ast::Statement::Sutra(sutra)
            },
            ast::Statement::Expr(expr) => {
                // TODO: Add span to expression if needed
                ast::Statement::Expr(expr)
            },
            ast::Statement::Block(block) => {
                // TODO: Add span to block if needed
                ast::Statement::Block(block)
            },
            item @ ast::Statement::Item(_) => item,
            _ => stmt,
        }
    }
    
    /// Parse a धर्म (dharma) declaration - class/type definition
    fn parse_dharma_decl(&mut self) -> ParseResult<ast::Statement> {
        let start_pos = self.current_span().map(|s| s.start).unwrap_or(0);
        self.expect(Token::Dharma)?;
        
        let name = self.parse_identifier()?;
        
        // Parse type parameters if any
        let type_params = if self.matches(Token::Less) {
            self.parse_type_parameters()?
        } else {
            Vec::new()
        };
        
        // Parse fields
        self.expect(Token::LBrace)?;
        let mut fields = Vec::new();
        
        while !self.matches(Token::RBrace) {
            let field_visibility = self.parse_visibility()?;
            let name = self.parse_identifier()?;
            self.expect(Token::Colon)?;
            let ty = self.parse_type()?;
            
            let default_value = if self.matches(Token::Equals) {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };
            
            self.expect(Token::Semicolon)?;
            
            fields.push(ast::FieldDef {
                name,
                ty,
                visibility: field_visibility,
                default_value,
                span: self.current_span().unwrap_or_else(|| ast::Span::dummy()),
            });
        }
        
        let end_pos = self.current_span().map(|s| s.end).unwrap_or(0);
        
        Ok(ast::Statement::Item(ast::Item::Dharma(ast::DharmaDef {
            name,
            type_params,
            fields,
            methods: Vec::new(), // Will be populated later
            visibility: ast::Visibility::Public, // Default to public for now
            span: ast::Span::new(start_pos, end_pos, 0), // 0 for main file
        })))
    }
    
    /// Parse a मन्त्र (mantra) declaration - function/method
    fn parse_mantra_decl(&mut self) -> ParseResult<ast::Statement> {
        let start_pos = self.current_span().map(|s| s.start).unwrap_or(0);
        let visibility = self.parse_visibility()?;
        
        self.expect(Token::Mantra)?;
        let name = self.parse_identifier()?;
        
        // Parse type parameters if any
        let type_params = if self.matches(Token::Less) {
            self.parse_type_parameters()?
        } else {
            Vec::new()
        };
        
        // Parse parameters
        self.expect(Token::LParen)?;
        let mut params = Vec::new();
        
        if !self.matches(Token::RParen) {
            loop {
                let param_name = self.parse_identifier()?;
                self.expect(Token::Colon)?;
                let param_ty = self.parse_type()?;
                
                params.push(ast::Param {
                    name: param_name,
                    ty: param_ty,
                    span: self.current_span().unwrap_or_else(|| ast::Span::dummy()),
                });
                
                if !self.matches(Token::Comma) {
                    self.expect(Token::RParen)?;
                    break;
                }
            }
        }
        
        // Parse return type
        let return_type = if self.matches(Token::Arrow) {
            self.parse_type()?
        } else {
            ast::Type::Unit
        };
        
        // Parse function body
        let body = self.parse_block()?;
        
        let end_pos = self.current_span().map(|s| s.end).unwrap_or(0);
        
        Ok(ast::Statement::Item(ast::Item::Mantra(ast::MantraDef {
            name,
            type_params,
            params,
            return_type,
            body,
            is_async: false, // Will be handled with async keyword later
            is_unsafe: false, // Will be handled with unsafe keyword later
            visibility,
            span: ast::Span::new(start_pos, end_pos, 0), // 0 for main file
        })))
    }
    
    /// Parse a सूत्र (sutra) declaration - constant/variable
    fn parse_sutra_decl(&mut self) -> ParseResult<ast::Statement> {
        let start_pos = self.current_span().map(|s| s.start).unwrap_or(0);
        let visibility = self.parse_visibility()?;
        
        self.expect(Token::Sutra)?;
        let is_mutable = self.matches(Token::Mut);
        
        // Parse pattern (for now, just a simple identifier pattern)
        let pattern = self.parse_pattern()?;
        
        // Parse type annotation if present
        let type_annotation = if self.matches(Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        // Parse initializer (required for sutra)
        self.expect(Token::Equals)?;
        let value = self.parse_expression()?;
        
        self.expect(Token::Semicolon)?;
        
        let end_pos = self.current_span().map(|s| s.end).unwrap_or(0);
        
        Ok(ast::Statement::Sutra(ast::SutraDef {
            pattern,
            type_annotation,
            value,
            is_static: false, // Will be handled with static keyword if needed
            span: ast::Span::new(start_pos, end_pos, 0), // 0 for main file
        }))
    }
    
    /// Parse a यन्त्र (yantra) declaration - module/namespace
    fn parse_yantra_decl(&mut self) -> ParseResult<ast::Statement> {
        let start_pos = self.current_span().map(|s| s.start).unwrap_or(0);
        let visibility = self.parse_visibility()?;
        
        self.expect(Token::Yantra)?;
        let name = self.parse_identifier()?;
        
        self.expect(Token::LBrace)?;
        let mut items = Vec::new();
        
        while !self.matches(Token::RBrace) {
            items.push(self.parse_item()?);
        }
        
        let end_pos = self.current_span().map(|s| s.end).unwrap_or(0);
        
        // Create a new scope for the yantra
        Ok(ast::Statement::Item(ast::Item::Yantra(ast::YantraDef {
            name,
            items,
            span: ast::Span::new(start_pos, end_pos, 0), // 0 for main file
        })))
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
    use vaaktra_lexer::Lexer;
    
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
