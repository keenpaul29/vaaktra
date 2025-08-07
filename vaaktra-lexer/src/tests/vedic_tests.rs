use super::*;

// Test Vedic Sanskrit keywords and their tokenization
#[test]
fn test_vedic_keywords() {
    let input = "धर्म मन्त्र सूत्र यदि अथवा यावत् प्रत्येक निर्गम अनुवृत्ति ऋत";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next().unwrap().0, Token::Class);
    assert_eq!(lexer.next().unwrap().0, Token::Fn);
    assert_eq!(lexer.next().unwrap().0, Token::Let);
    assert_eq!(lexer.next().unwrap().0, Token::If);
    assert_eq!(lexer.next().unwrap().0, Token::Else);
    assert_eq!(lexer.next().unwrap().0, Token::While);
    assert_eq!(lexer.next().unwrap().0, Token::ForEach);
    assert_eq!(lexer.next().unwrap().0, Token::Break);
    assert_eq!(lexer.next().unwrap().0, Token::Continue);
    assert_eq!(lexer.next().unwrap().0, Token::Const);
}

// Test Vedic type system
#[test]
fn test_vedic_types() {
    let input = "सङ्ख्या सत्यासत्य शब्द सूची निधान शून्य";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next().unwrap().0, Token::NumberType);
    assert_eq!(lexer.next().unwrap().0, Token::BoolType);
    assert_eq!(lexer.next().unwrap().0, Token::StringType);
    assert_eq!(lexer.next().unwrap().0, Token::ListType);
    assert_eq!(lexer.next().unwrap().0, Token::MapType);
    assert_eq!(lexer.next().unwrap().0, Token::Void);
}

// Test Vedic literals
#[test]
fn test_vedic_literals() {
    let input = r#"सत्य असत्य "नमस्ते" १२३४"#;
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next().unwrap().0, Token::True);
    assert_eq!(lexer.next().unwrap().0, Token::False);
    
    if let (Token::StringLit(s), _) = lexer.next().unwrap() {
        assert_eq!(s, "नमस्ते");
    } else {
        panic!("Expected string literal");
    }
    
    if let (Token::Number(n), _) = lexer.next().unwrap() {
        assert_eq!(n, 1234);
    } else {
        panic!("Expected number literal");
    }
}

// Test Vedic operators
#[test]
fn test_vedic_operators() {
    let input = "= == != < <= > >= + - * / % && || !";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next().unwrap().0, Token::Equals);
    assert_eq!(lexer.next().unwrap().0, Token::Eq);
    assert_eq!(lexer.next().unwrap().0, Token::Neq);
    assert_eq!(lexer.next().unwrap().0, Token::Lt);
    assert_eq!(lexer.next().unwrap().0, Token::Le);
    assert_eq!(lexer.next().unwrap().0, Token::Gt);
    assert_eq!(lexer.next().unwrap().0, Token::Ge);
    assert_eq!(lexer.next().unwrap().0, Token::Plus);
    assert_eq!(lexer.next().unwrap().0, Token::Minus);
    assert_eq!(lexer.next().unwrap().0, Token::Star);
    assert_eq!(lexer.next().unwrap().0, Token::Slash);
    assert_eq!(lexer.next().unwrap().0, Token::Percent);
    assert_eq!(lexer.next().unwrap().0, Token::And);
    assert_eq!(lexer.next().unwrap().0, Token::Or);
    assert_eq!(lexer.next().unwrap().0, Token::Not);
}

// Test Vedic delimiters
#[test]
fn test_vedic_delimiters() {
    let input = "() {} [] ; , . :";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next().unwrap().0, Token::LParen);
    assert_eq!(lexer.next().unwrap().0, Token::RParen);
    assert_eq!(lexer.next().unwrap().0, Token::LBrace);
    assert_eq!(lexer.next().unwrap().0, Token::RBrace);
    assert_eq!(lexer.next().unwrap().0, Token::LBracket);
    assert_eq!(lexer.next().unwrap().0, Token::RBracket);
    assert_eq!(lexer.next().unwrap().0, Token::Semicolon);
    assert_eq!(lexer.next().unwrap().0, Token::Comma);
    assert_eq!(lexer.next().unwrap().0, Token::Dot);
    assert_eq!(lexer.next().unwrap().0, Token::Colon);
}

// Test Vedic concepts
#[test]
fn test_vedic_concepts() {
    let input = "ब्रह्मन् आत्मन् संस्कार";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next().unwrap().0, Token::Object);
    assert_eq!(lexer.next().unwrap().0, Token::SelfValue);
    assert_eq!(lexer.next().unwrap().0, Token::Cast);
}

// Test Devanagari identifiers
#[test]
fn test_devanagari_identifiers() {
    let input = "परिवर्तन_१२३ परिवर्तन_४५६";
    let mut lexer = Lexer::new(input);
    
    if let (Token::Ident(s), _) = lexer.next().unwrap() {
        assert_eq!(s, "परिवर्तन_१२३");
    } else {
        panic!("Expected identifier");
    }
    
    if let (Token::Ident(s), _) = lexer.next().unwrap() {
        assert_eq!(s, "परिवर्तन_४५६");
    } else {
        panic!("Expected identifier");
    }
}
