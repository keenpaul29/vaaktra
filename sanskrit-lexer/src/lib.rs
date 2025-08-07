//! Lexer for the Sanskrit programming language
//! Tokenizes source code into a stream of tokens for the parser

use logos::{Logos, SpannedIter};
use std::fmt;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\r\f]+")] // Skip whitespace
#[logos(skip r"//[^\n]*")] // Skip line comments
pub enum Token {
    // Keywords (Sanskrit-based)
    #[token("प्रधानं")]
    Main,
    
    // Control flow
    #[token("यदि")]
    If,
    #[token("अन्यथा")]
    Else,
    #[token("यावत्")]
    While,
    
    // Types
    #[token("संख्या")]
    Int,
    #[token("सत्यासत्य")]
    Bool,
    #[token("पाठ")]
    String,
    
    // Literals
    #[regex(r"[०-९]+", |lex| lex.slice().parse().ok())]
    Number(i64),
    #[token("सत्यम्")]
    True,
    #[token("मिथ्या")]
    False,
    
    // Identifiers
    #[regex(r"[\p{Script=Devanagari}\p{Nd}_][\p{Script=Devanagari}\p{Nd}_]*", |lex| lex.slice().to_string())]
    Ident(String),
    
    // Operators
    #[token("=")]
    Equals,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    
    // Delimiters
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(";")]
    Semicolon,
    
    // Error token for invalid input
    #[error]
    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Lexer<'a> {
    inner: SpannedIter<'a, Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            inner: Token::lexer(input).spanned(),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = (Token, std::ops::Range<usize>);
    
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lex_keywords() {
        let input = "प्रधानं यदि अन्यथा यावत्";
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next().unwrap().0, Token::Main);
        assert_eq!(lexer.next().unwrap().0, Token::If);
        assert_eq!(lexer.next().unwrap().0, Token::Else);
        assert_eq!(lexer.next().unwrap().0, Token::While);
    }
    
    #[test]
    fn test_lex_numbers() {
        let input = "१ २ ३ ४ ५६७";
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next().unwrap().0, Token::Number(1));
        assert_eq!(lexer.next().unwrap().0, Token::Number(2));
        assert_eq!(lexer.next().unwrap().0, Token::Number(3));
        assert_eq!(lexer.next().unwrap().0, Token::Number(4));
        assert_eq!(lexer.next().unwrap().0, Token::Number(567));
    }
}
