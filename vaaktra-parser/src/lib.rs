//! Parser for the Sanskrit programming language
//! Converts tokens into an Abstract Syntax Tree (AST)

pub mod ast;

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
        let mut items = Vec::new();
        
        while self.peek().is_some() {
            let stmt = self.parse_statement()?;
            if let ast::Statement::Item(item) = stmt {
                items.push(item);
            } else {
                // For now, wrap non-item statements in a Praarabdha block
                items.push(ast::Item::Praarabdha(vec![stmt]));
            }
        }
        
        Ok(ast::Program { items, span: Span::dummy() })
    }
    
    /// Parse a statement
    fn parse_statement(&mut self) -> ParseResult<ast::Statement> {
        let _start_pos = self.current_span().map(|s| s.start).unwrap_or(0);
        
        let stmt = match self.peek() {
            // Vedic-inspired keywords
            Some(Token::Class) => {
                // Class declarations are top-level items, wrap in an Item statement
                if let ast::Statement::Item(ast::Item::Dharma(dharma)) = self.parse_dharma_decl()? {
                    ast::Statement::Item(ast::Item::Dharma(dharma))
                } else {
                    return Err(ParseError::SyntaxError("Expected dharma declaration".to_string()));
                }
            },
            Some(Token::Fn) => {
                // Function declarations are top-level items, wrap in an Item statement
                if let ast::Statement::Item(ast::Item::Mantra(mantra)) = self.parse_mantra_decl()? {
                    ast::Statement::Item(ast::Item::Mantra(mantra))
                } else {
                    return Err(ParseError::SyntaxError("Expected mantra declaration".to_string()));
                }
            },
            Some(Token::Let) => {
                // Variable declarations are statements
                self.parse_sutra_decl()?
            },
            
            // Control flow - for now, return placeholder statements
            Some(Token::If) => ast::Statement::Shunya, // TODO: implement parse_if_statement
            Some(Token::While) => ast::Statement::Shunya, // TODO: implement parse_while_statement  
            
            // Blocks and expressions - for now, return placeholder statements
            Some(Token::LBrace) => ast::Statement::Shunya, // TODO: implement parse_block
            _ => ast::Statement::Shunya, // TODO: implement parse_expression_statement
        };
        
        let _end_pos = self.current_span().map(|s| s.end).unwrap_or(0);
        
        // Return the statement as-is for now
        Ok(stmt)
    }
    
    /// Parse a धर्म (dharma) declaration - class/type definition
    fn parse_dharma_decl(&mut self) -> ParseResult<ast::Statement> {
        let start_pos = self.current_span().map(|s| s.start).unwrap_or(0);
        self.expect(Token::Class)?;
        
        let name = self.parse_identifier()?;
        
        // Parse type parameters if any - for now, skip
        let type_params = Vec::new(); // TODO: implement parse_type_parameters
        
        // Parse fields - for now, create empty fields
        self.expect(Token::LBrace)?;
        let fields = Vec::new(); // TODO: implement field parsing
        
        // Skip to closing brace for now
        while !self.matches(Token::RBrace) {
            self.next(); // Skip tokens until we find closing brace
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
        let visibility = ast::Visibility::Public; // TODO: implement parse_visibility
        
        self.expect(Token::Fn)?;
        let name = self.parse_identifier()?;
        
        // Parse type parameters if any - for now, skip
        let type_params = Vec::new(); // TODO: implement parse_type_parameters
        
        // Parse parameters - for now, create empty params
        self.expect(Token::LParen)?;
        let params = Vec::new(); // TODO: implement parameter parsing
        
        // Skip to closing paren for now
        while !self.matches(Token::RParen) {
            self.next(); // Skip tokens until we find closing paren
        }
        
        // Parse return type - for now, use a placeholder
        let return_type = ast::Type::Infer(ast::Span::dummy()); // TODO: implement return type parsing
        
        // Parse function body - for now, create empty block
        let body = ast::Block {
            stmts: Vec::new(),
            expr: None,
            span: ast::Span::dummy(),
        }; // TODO: implement parse_block
        
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
        
        self.expect(Token::Let)?;
        let is_mutable = false; // TODO: Add mutable token support
        
        // Parse pattern (for now, just a simple identifier pattern)
        let pattern = ast::Pattern::Bind {
            name: self.parse_identifier()?,
            mutable: is_mutable,
            by_ref: false,
            subpattern: None,
            span: ast::Span::dummy(),
        }; // TODO: implement parse_pattern
        
        // Parse type annotation if present
        let type_annotation = if self.matches(Token::Colon) {
            Some(ast::Type::Infer(ast::Span::dummy())) // TODO: implement parse_type
        } else {
            None
        };
        
        // Parse initializer (required for sutra)
        self.expect(Token::Equals)?;
        let value = ast::Expr::Error(ast::Span::dummy()); // TODO: implement parse_expression
        
        self.expect(Token::Semicolon)?;
        
        let end_pos = self.current_span().map(|s| s.end).unwrap_or(0);
        
        Ok(ast::Statement::Sutra(ast::SutraDef {
            pattern,
            type_annotation,
            value,
            is_mutable,
            is_static: false, // Will be handled with static keyword if needed
            span: ast::Span::new(start_pos, end_pos, 0), // 0 for main file
        }))
    }
    
    // TODO: Implement yantra (module) parsing when needed
    
    /// Get the current span
    fn current_span(&self) -> Option<std::ops::Range<usize>> {
        self.current_span.clone()
    }
    
    /// Parse an identifier
    fn parse_identifier(&mut self) -> ParseResult<ast::RcStr> {
        match self.next() {
            Some(Token::Ident(name)) => Ok(ast::RcStr::new(&name)),
            found => {
                let span = self.current_span.clone().unwrap_or(0..0);
                Err(ParseError::UnexpectedToken {
                    expected: "identifier".to_string(),
                    found,
                    span: (span.start, span.end),
                })
            }
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
    use vaaktra_lexer::Lexer;
    
    #[test]
    fn test_parse_empty_program() {
        let input = "";
        let tokens = Lexer::new(input);
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        assert!(program.items.is_empty());  
    }
    
    // More tests will be added as we implement more parsing functionality
}
