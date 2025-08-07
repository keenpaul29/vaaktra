//! Parser for the Vāktra (वाक्त्र) programming language
//! 
//! This module implements a recursive descent parser that converts token streams
//! into an Abstract Syntax Tree (AST) representing the program's structure.

use logos::Span;
use crate::ast::*;
use crate::lexer::{Token, TokenKind};
use std::iter::Peekable;
use std::slice::Iter;

/// Parser for Vāktra source code
pub struct Parser<'a> {
    /// Tokens to parse
    tokens: Peekable<Iter<'a, Token>>,
    /// Current position in the source
    current: usize,
    /// Source code for error reporting
    source: &'a str,
}

/// Parser error type
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected:?}, found {found:?}")]
    UnexpectedToken {
        expected: String,
        found: TokenKind,
        span: Span,
    },
    
    #[error("Unexpected end of input")]
    UnexpectedEof,
    
    #[error("Expected {0}")]
    Expected(&'static str, Span),
    
    #[error("Invalid numeric literal: {0}")]
    InvalidNumber(String, Span),
    
    #[error("Invalid string literal: {0}")]
    InvalidString(String, Span),
}

type ParseResult<T> = Result<T, ParseError>;

impl<'a> Parser<'a> {
    /// Create a new parser for the given tokens and source
    pub fn new(tokens: &'a [Token], source: &'a str) -> Self {
        Self {
            tokens: tokens.iter().peekable(),
            current: 0,
            source,
        }
    }
    
    /// Parse the entire program
    pub fn parse_program(&mut self) -> ParseResult<Vec<Item>> {
        let mut items = Vec::new();
        
        while !self.is_at_end() {
            if let Some(item) = self.declaration()? {
                items.push(item);
            } else {
                // Skip any unexpected tokens
                self.advance();
            }
        }
        
        Ok(items)
    }
    
    /// Parse a declaration (function, variable, etc.)
    fn declaration(&mut self) -> ParseResult<Option<Item>> {
        Ok(match self.peek() {
            Some(token) => match token.kind {
                // Match Vedic-inspired keywords first
                TokenKind::Dharma => Some(Item::Dharma(self.dharma_decl()?)),
                TokenKind::Mantra => Some(Item::Mantra(self.mantra_decl()?)),
                TokenKind::Sutra => Some(Item::Sutra(self.sutra_decl()?)),
                TokenKind::Yantra => Some(Item::Yantra(self.yantra_decl()?)),
                // Keep backward compatibility with English keywords for now
                TokenKind::Class => Some(Item::Dharma(self.dharma_decl()?)),
                TokenKind::Fn => Some(Item::Mantra(self.mantra_decl()?)),
                TokenKind::Let => Some(Item::Sutra(self.sutra_decl()?)),
                _ => {
                    // Try to parse an expression statement
                    let expr = self.expression()?;
                    self.consume(TokenKind::Semicolon, "Expected ';' after expression")?;
                    Some(Item::Expr(expr))
                }
            },
            None => None,
        })
    }
    
    /// Parse a mantra (function) declaration
    fn mantra_decl(&mut self) -> ParseResult<MantraDef> {
        // Consume either 'mantra' or 'fn' keyword
        if !self.consume_any(&[TokenKind::Mantra, TokenKind::Fn]).is_some() {
            return self.error("Expected 'मन्त्र' (mantra) or 'fn' keyword to begin function declaration");
        }
        
        let name = if let Some(ident) = self.consume_any(&[TokenKind::Identifier]) {
            ident.lexeme.clone()
        } else {
            return Err(ParseError::Expected("function name", self.peek_span()));
        };
        
        // Parse type parameters
        let type_params = if self.check(TokenKind::Lt) {
            self.type_parameters()?
        } else {
            Vec::new()
        };
        
        // Parse parameters
        self.consume(TokenKind::LParen, "Expected '(' after function name")?;
        let mut params = Vec::new();
        
        if !self.check(TokenKind::RParen) {
            loop {
                if self.check(TokenKind::RParen) {
                    break;
                }
                
                let pattern = self.pattern()?;
                self.consume(TokenKind::Colon, "Expected ':' after parameter name")?;
                let ty = self.type_annotation()?;
                
                params.push(Param {
                    pattern,
                    ty,
                    default_value: None,
                    span: self.peek_span(),
                });
                
                if !self.consume_any(&[TokenKind::Comma]).is_some() {
                    break;
                }
            }
        }
        
        self.consume(TokenKind::RParen, "Expected ')' after parameters")?;
        
        // Parse return type
        let return_ty = if self.consume_any(&[TokenKind::ThinArrow]).is_some() {
            self.type_annotation()?
        } else {
            Type::Tuple(Vec::new(), self.peek_span())
        };
        
        // Parse mantra body
        let body = self.block()?;
        
        Ok(MantraDef {
            name: RcStr::from(name),
            type_params,
            params,
            return_ty,
            body,
            is_async: false,  // TODO: Add async support
            is_unsafe: false, // TODO: Add unsafe support
            span: self.peek_span(),
        })
    }
    
    /// Parse a let declaration
    fn let_decl(&mut self) -> ParseResult<LetDecl> {
        self.consume(TokenKind::Let, "Expected 'let' keyword")?;
        let is_mut = self.consume_any(&[TokenKind::Mut]).is_some();
        
        let pattern = self.pattern()?;
        let ty = if self.consume_any(&[TokenKind::Colon]).is_some() {
            Some(self.type_annotation()?)
        } else {
            None
        };
        
        let initializer = if self.consume_any(&[TokenKind::Eq]).is_some() {
            Some(Box::new(self.expression()?))
        } else {
            None
        };
        
        self.consume(TokenKind::Semicolon, "Expected ';' after let declaration")?;
        
        Ok(LetDecl {
            pattern,
            ty,
            initializer,
            is_mut,
            span: self.peek_span(),
        })
    }
    
    /// Parse a dharma (class/type) declaration
    fn dharma_decl(&mut self) -> ParseResult<DharmaDef> {
        // Consume either 'dharma' or 'class' keyword
        if !self.consume_any(&[TokenKind::Dharma, TokenKind::Class]).is_some() {
            return Err(ParseError::Expected("'dharma' or 'class' keyword", self.peek_span()));
        }
        
        let name = if let Some(ident) = self.consume_any(&[TokenKind::Identifier]) {
            RcStr::from(ident.lexeme.clone())
        } else {
            return Err(ParseError::Expected("dharma name", self.peek_span()));
        };
        
        // Parse type parameters
        let type_params = if self.check(TokenKind::Lt) {
            self.type_parameters()?
        } else {
            Vec::new()
        };
        
        // Parse fields and methods
        self.consume(TokenKind::LBrace, "Expected '{' to start dharma body")?;
        
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            // Parse field or method
            if self.match_any(&[TokenKind::Mantra, TokenKind::Fn]) {
                methods.push(self.mantra_decl()?);
            } else {
                // Parse field declaration
                let name = if let Some(ident) = self.consume_any(&[TokenKind::Identifier]) {
                    RcStr::from(ident.lexeme.clone())
                } else {
                    return Err(ParseError::Expected("field name", self.peek_span()));
                };
                
                self.consume(TokenKind::Colon, "Expected ':' after field name")?;
                let ty = self.type_annotation()?;
                
                fields.push(FieldDef {
                    name,
                    ty,
                    visibility: Visibility::Public, // Default to public
                    span: self.peek_span(),
                });
                
                if !self.consume_any(&[TokenKind::Comma, TokenKind::Semicolon]).is_some() {
                    break;
                }
            }
        }
        
        self.consume(TokenKind::RBrace, "Expected '}' to end dharma body")?;
        
        Ok(DharmaDef {
            name,
            type_params,
            fields,
            methods,
            span: self.peek_span(),
        })
    }
    
    /// Parse a sutra (variable/constant) declaration
    fn sutra_decl(&mut self) -> ParseResult<SutraDef> {
        // Consume either 'sutra' or 'let' keyword
        let is_mutable = if self.consume_any(&[TokenKind::Sutra]).is_some() {
            // 'sutra' is immutable by default
            false
        } else if self.consume_any(&[TokenKind::Let]).is_some() {
            // 'let' can be mutable with 'mut' keyword
            self.consume_any(&[TokenKind::Mut]).is_some()
        } else {
            return Err(ParseError::Expected("'sutra' or 'let' keyword", self.peek_span()));
        };
        
        let pattern = self.pattern()?;
        let type_annotation = if self.consume_any(&[TokenKind::Colon]).is_some() {
            Some(self.type_annotation()?)
        } else {
            None
        };
        
        self.consume(TokenKind::Eq, "Expected '=' in sutra declaration")?;
        
        let value = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expected ';' after sutra declaration")?;
        
        Ok(SutraDef {
            pattern,
            type_annotation,
            value,
            is_mutable,
            is_static: false, // TODO: Add support for static sutras
            span: self.peek_span(),
        })
    }
    
    /// Parse a yantra (module/namespace) declaration
    fn yantra_decl(&mut self) -> ParseResult<YantraDef> {
        self.consume(TokenKind::Yantra, "Expected 'yantra' keyword")?;
        
        let name = if let Some(ident) = self.consume_any(&[TokenKind::Identifier]) {
            RcStr::from(ident.lexeme.clone())
        } else {
            return Err(ParseError::Expected("yantra name", self.peek_span()));
        };
        
        self.consume(TokenKind::LBrace, "Expected '{' to start yantra body")?;
        
        let mut items = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            if let Some(item) = self.declaration()? {
                items.push(item);
            } else {
                self.advance();
            }
        }
        
        self.consume(TokenKind::RBrace, "Expected '}' to end yantra body")?;
        
        Ok(YantraDef {
            name,
            items,
            span: self.peek_span(),
        })
    }
    
    /// Parse a constant declaration (for backward compatibility)
    fn const_decl(&mut self) -> ParseResult<ConstDecl> {
        self.consume(TokenKind::Const, "Expected 'const' keyword")?;
        
        let name = if let Some(ident) = self.consume_any(&[TokenKind::Identifier]) {
            ident.lexeme.clone()
        } else {
            return Err(ParseError::Expected("constant name", self.peek_span()));
        };
        
        self.consume(TokenKind::Colon, "Expected ':' after constant name")?;
        let ty = self.type_annotation()?;
        self.consume(TokenKind::Eq, "Expected '=' in constant declaration")?;
        
        let value = Box::new(self.expression()?);
        self.consume(TokenKind::Semicolon, "Expected ';' after constant value")?;
        
        Ok(ConstDecl {
            name,
            ty,
            value,
            span: self.peek_span(),
        })
    }
    
    /// Parse a type declaration
    fn type_decl(&mut self) -> ParseResult<TypeDecl> {
        self.consume(TokenKind::Type, "Expected 'type' keyword")?;
        
        let name = if let Some(ident) = self.consume_any(&[TokenKind::Identifier]) {
            ident.lexeme.clone()
        } else {
            return Err(ParseError::Expected("type name", self.peek_span()));
        };
        
        // Parse type parameters
        let type_params = if self.check(TokenKind::Lt) {
            self.type_parameters()?
        } else {
            Vec::new()
        };
        
        self.consume(TokenKind::Eq, "Expected '=' in type declaration")?;
        
        let ty = self.type_annotation()?;
        self.consume(TokenKind::Semicolon, "Expected ';' after type declaration")?;
        
        Ok(TypeDecl {
            name,
            type_params,
            ty,
            span: self.peek_span(),
        })
    }
    
    /// Check if we've reached the end of the token stream
    fn is_at_end(&self) -> bool {
        self.peek().is_none()
    }
    
    /// Get the current span for error reporting
    fn current_span(&self) -> Span {
        self.peek().map(|t| t.span).unwrap_or_else(|| 0..0)
    }
    
    /// Create a parse error with Vedic-inspired message
    fn error<T>(&self, message: &str) -> ParseResult<T> {
        Err(ParseError::SyntaxError(
            format!("अशुद्धिः (Error): {}", message),
            self.current_span()
        ))
    }
    
    /// Get the next token without consuming it
    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek().copied()
    }
    
    /// Get the next token and consume it
    fn advance(&mut self) -> Option<&'a Token> {
        self.current += 1;
        self.tokens.next()
    }
    
    /// Check if the next token matches the expected kind
    fn check(&mut self, kind: TokenKind) -> bool {
        match self.peek() {
            Some(token) => token.kind == kind,
            None => false,
        }
    }
    
    /// Consume the next token if it matches the expected kind with Vedic-inspired messages
    fn consume(&mut self, kind: TokenKind, context: &str) -> ParseResult<&Token> {
        if self.check(kind) {
            Ok(self.advance().unwrap())
        } else {
            let found = self.peek().map(|t| t.kind).unwrap_or(TokenKind::Eof);
            let span = self.current_span();
            
            // Map token kinds to Vedic terms for better error messages
            let expected = match kind {
                TokenKind::Dharma => "'धर्म' (dharma/class)",
                TokenKind::Mantra => "'मन्त्र' (mantra/function)",
                TokenKind::Sutra => "'सूत्र' (sutra/variable)",
                TokenKind::Yantra => "'यन्त्र' (yantra/module)",
                TokenKind::LBrace => "'{' (वामावर्तः/left brace)",
                TokenKind::RBrace => "'}' (दक्षिणावर्तः/right brace)",
                TokenKind::LParen => "'(' (वामकोष्ठकः/left parenthesis)",
                TokenKind::RParen => "')' (दक्षिणकोष्ठकः/right parenthesis)",
                TokenKind::Semicolon => "';' (अर्धविरामः/semicolon)",
                TokenKind::Colon => "':'",
                TokenKind::Eq => "'=' (समः/equals)",
                TokenKind::Comma => "',' (अल्पविरामः/comma)",
                _ => "expected token"
            };
            
            let context_msg = if !context.is_empty() {
                format!(" in {}", context)
            } else {
                String::new()
            };
            
            Err(ParseError::UnexpectedToken {
                expected: format!("{} {}", expected, context_msg).trim().to_string(),
                found,
                span,
            })
        }
    }
    
    /// Check if the next token matches any of the given kinds
    fn match_any(&mut self, kinds: &[TokenKind]) -> bool {
        if let Some(token) = self.peek() {
            kinds.contains(&token.kind)
        } else {
            false
        }
    }
    
    /// Consume the next token if it matches any of the given kinds
    fn consume_any(&mut self, kinds: &[TokenKind]) -> Option<&Token> {
        if self.match_any(kinds) {
            self.advance()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{Lexer, Token};
    use crate::ast::*;
    
    fn parse_source(source: &str) -> ParseResult<Vec<Item>> {
        let tokens: Vec<Token> = Lexer::new(source).collect();
        let mut parser = Parser::new(&tokens, source);
        parser.parse_program()
    }
    
    #[test]
    fn test_parse_dharma_declaration() {
        let source = r#"
        धर्म पशुः {
            नाम: सङ्ख्या;
            
            मन्त्र नाम_प्राप्तिः() -> सङ्ख्या {
                यदि (सत्य) {
                    निर्गम;
                }
                यावत् (सत्य) {
                    नाम = १०;
                }
                नाम
            }
        }
        "#;
        
        let result = parse_source(source);
        assert!(result.is_ok(), "Failed to parse dharma: {:?}", result.err());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        
        if let Item::Dharma(dharma) = &items[0] {
            assert_eq!(dharma.name.as_str(), "पशुः");
            assert_eq!(dharma.fields.len(), 1);
            assert_eq!(dharma.methods.len(), 1);
            
            if let Type::Named(name, _) = &dharma.fields[0].ty {
                assert_eq!(name.as_str(), "सङ्ख्या");
            } else {
                panic!("Expected named type for field");
            }
            
            assert_eq!(dharma.methods[0].name.as_str(), "नाम_प्राप्तिः");
        } else {
            panic!("Expected a Dharma item");
        }
    }
    
    #[test]
    fn test_parse_mantra_declaration() {
        let source = r#"
        मन्त्र योगः(अ: सङ्ख्या, ब: सङ्ख्या) -> सङ्ख्या {
            अ + ब
        }
        "#;
        
        let result = parse_source(source);
        assert!(result.is_ok(), "Failed to parse mantra: {:?}", result.err());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        
        if let Item::Mantra(mantra) = &items[0] {
            assert_eq!(mantra.name.as_str(), "योगः");
            assert_eq!(mantra.params.len(), 2);
            assert_eq!(mantra.return_ty, Type::Named(RcStr::from("सङ्ख्या"), Span::dummy()));
        } else {
            panic!("Expected a Mantra item");
        }
    }
    
    #[test]
    fn test_parse_sutra_declaration() {
        let source = r#"
        सूत्र नाम = ४२;
        "#;
        
        let result = parse_source(source);
        assert!(result.is_ok(), "Failed to parse sutra: {:?}", result.err());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        
        if let Item::Sutra(sutra) = &items[0] {
            assert!(!sutra.is_mutable, "Sutra should be immutable by default");
            if let Pattern::Ident(ident) = &sutra.pattern {
                assert_eq!(ident.name.as_str(), "नाम");
            } else {
                panic!("Expected identifier pattern");
            }
        } else {
            panic!("Expected a Sutra item");
        }
    }
    
    #[test]
    fn test_parse_yantra_declaration() {
        let source = r#"
        यन्त्र गणितम् {
            सूत्र पाई = 3.14159;
            
            मन्त्र वर्गः(अ: सङ्ख्या) -> सङ्ख्या {
                अ * अ
            }
        }
        "#;
        
        let result = parse_source(source);
        assert!(result.is_ok(), "Failed to parse yantra: {:?}", result.err());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        
        if let Item::Yantra(yantra) = &items[0] {
            assert_eq!(yantra.name.as_str(), "गणितम्");
            assert_eq!(yantra.items.len(), 2);
            
            if let Item::Sutra(sutra) = &yantra.items[0] {
                assert_eq!(sutra.pattern.to_string(), "पाई");
            } else {
                panic!("Expected a Sutra inside Yantra");
            }
            
            if let Item::Mantra(mantra) = &yantra.items[1] {
                assert_eq!(mantra.name.as_str(), "वर्गः");
            } else {
                panic!("Expected a Mantra inside Yantra");
            }
        } else {
            panic!("Expected a Yantra item");
        }
    }
    
    #[test]
    fn test_parse_control_flow() {
        let source = r#"
        मन्त्र नियन्त्रण_परीक्षा(अ: सङ्ख्या) -> सङ्ख्या {
            यदि (अ > ०) {
                निर्गम १;
            } अन्यथा यदि (अ < ०) {
                निर्गम -१;
            } अन्यथा {
                निर्गम ०;
            }
            
            सूत्र फलम् = १;
            यावत् (फलम् < १०) {
                फलम् = फलम् * २;
            }
            
            फलम्
        }
        "#;
        
        let result = parse_source(source);
        assert!(result.is_ok(), "Failed to parse control flow: {:?}", result.err());
    }
    
    #[test]
    fn test_empty_program() {
        let source = "";
        let tokens: Vec<_> = Lexer::new(source).collect();
        let mut parser = Parser::new(&tokens, source);
        let program = parser.parse_program().unwrap();
        assert!(program.is_empty());
    }
    
    // More tests will be added as we implement more parser functionality
}
