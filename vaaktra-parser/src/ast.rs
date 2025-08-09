//! Abstract Syntax Tree (AST) for the Vāktra (वाक्त्र) programming language
//! Deeply integrated with Vedic Sanskrit concepts and optimized for performance

use std::fmt;
use std::sync::Arc;
use std::hash::{Hash, Hasher};

/// Represents a location in the source code with file information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub file_id: u32,
}

impl Span {
    pub fn new(start: usize, end: usize, file_id: u32) -> Self {
        debug_assert!(start <= end, "Invalid span: start > end");
        Span { start, end, file_id }
    }
    
    pub fn dummy() -> Self {
        Span { start: 0, end: 0, file_id: 0 }
    }
}

/// A node in the AST with source location information
/// Uses Arc for efficient cloning of large AST subtrees
#[derive(Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
    pub source: Option<Arc<str>>,  // Optional source code for better error messages
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span, source: Option<Arc<str>>) -> Self {
        Spanned { node, span, source }
    }
    
    pub fn with_dummy_span(node: T) -> Self {
        Spanned {
            node,
            span: Span::dummy(),
            source: None,
        }
    }
}

// Manual implementations to avoid deriving on T
impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node && self.span == other.span
    }
}

impl<T: Eq> Eq for Spanned<T> {}

impl<T: Hash> Hash for Spanned<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node.hash(state);
        self.span.hash(state);
    }
}

impl<T: fmt::Debug> fmt::Debug for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} @ {:?}", self.node, self.span)
    }
}

/// Reference-counted string for identifiers and literals
/// More memory efficient than String for repeated values
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RcStr(Arc<str>);

impl RcStr {
    pub fn new(s: &str) -> Self {
        RcStr(Arc::from(s))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RcStr {
    fn from(s: &str) -> Self {
        RcStr::new(s)
    }
}

impl From<String> for RcStr {
    fn from(s: String) -> Self {
        RcStr(Arc::from(s))
    }
}

impl std::ops::Deref for RcStr {
    type Target = str;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for RcStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ===== Core AST Nodes =====

/// The root of the AST, representing an entire program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: Span,
}

/// Top-level items in a Vāktra program
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// A धर्म (dharma) - class/type definition
    Dharma(DharmaDef),
    /// A मन्त्र (mantra) - function definition
    Mantra(MantraDef),
    /// A सूत्र (sūtra) - constant or variable definition
    Sutra(SutraDef),
    /// A यन्त्र (yantra) - module/namespace
    Yantra(YantraDef),
    /// A प्रारब्ध (prārabdha) - initialization block
    Praarabdha(Vec<Statement>),
}

/// A धर्म (dharma) represents a class or type definition
#[derive(Debug, Clone, PartialEq)]
pub struct DharmaDef {
    pub name: RcStr,
    pub type_params: Vec<TypeParam>,
    pub fields: Vec<FieldDef>,
    pub methods: Vec<MantraDef>,
    pub visibility: Visibility,
    pub span: Span,
}

/// A मन्त्र (mantra) represents a function or method
#[derive(Debug, Clone, PartialEq)]
pub struct MantraDef {
    pub name: RcStr,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Block,
    pub is_async: bool,
    pub is_unsafe: bool,
    pub visibility: Visibility,
    pub span: Span,
}

/// A सूत्र (sūtra) represents a constant or variable
#[derive(Debug, Clone, PartialEq)]
pub struct SutraDef {
    pub pattern: Pattern,
    pub type_annotation: Option<Type>,
    pub value: Expr,
    pub is_mutable: bool,
    pub is_static: bool,
    pub span: Span,
}

/// A यन्त्र (yantra) represents a module or namespace
#[derive(Debug, Clone, PartialEq)]
pub struct YantraDef {
    pub name: RcStr,
    pub items: Vec<Item>,
    pub span: Span,
}

/// Statements in Vāktra
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// A top-level item
    Item(Item),
    /// A variable declaration
    Sutra(SutraDef),
    /// An expression statement
    Expr(Box<Expr>),
    /// A block of statements
    Block(Block),
    /// If-else statement (यदि-अथवा)
    Yadi {
        condition: Box<Expr>,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    /// While loop (यावत्)
    Yaavat {
        condition: Box<Expr>,
        body: Box<Statement>,
    },
    /// For-each loop (प्रत्येक)
    Pratyeka {
        pattern: Pattern,
        iterable: Box<Expr>,
        body: Box<Statement>,
    },
    /// Return statement (प्रत्याहर)
    Pratyahara(Option<Box<Expr>>),
    /// Break statement (निर्गम)
    Nirgama(Option<Box<Expr>>),
    /// Continue statement (अनुवृत्ति)
    Anuvrtti,
    /// Empty statement (शून्य)
    Shunya,
}

// ===== Patterns and Types =====

/// Pattern matching in Vāktra
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Wildcard pattern (_)
    Any(Span),
    /// Literal pattern (42, true, "string")
    Literal(Literal),
    /// Variable binding (x, mut y, ref z)
    Bind {
        name: RcStr,
        mutable: bool,
        by_ref: bool,
        subpattern: Option<Box<Pattern>>,
        span: Span,
    },
    /// Tuple pattern (a, b, c)
    Tuple(Vec<Pattern>, Span),
    /// Struct pattern (Point { x, y })
    Struct {
        path: Path,
        fields: Vec<FieldPattern>,
        rest: bool,
        span: Span,
    },
}

/// Field pattern in a struct pattern
#[derive(Debug, Clone, PartialEq)]
pub struct FieldPattern {
    pub name: RcStr,
    pub pattern: Pattern,
    pub span: Span,
}

/// Type parameter with constraints
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    pub name: RcStr,
    pub bounds: Vec<TypeBound>,
    pub default: Option<Type>,
    pub span: Span,
}

/// Type bound for generic parameters
#[derive(Debug, Clone, PartialEq)]
pub struct TypeBound {
    pub bound: Path,
    pub span: Span,
}

/// Field definition in a struct or class
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDef {
    pub name: RcStr,
    pub ty: Type,
    pub default_value: Option<Expr>,
    pub visibility: Visibility,
    pub span: Span,
}

/// Visibility modifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Crate,
    Super,
    In(RcStr),  // Path to parent module
}

/// Function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: RcStr,
    pub ty: Type,
    pub default_value: Option<Expr>,
    pub span: Span,
}

/// Block of statements with optional tail expression
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Statement>,
    pub expr: Option<Box<Expr>>,
    pub span: Span,
}

/// Types in Vāktra
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Named type (सङ्ख्या, सत्यासत्य, etc.)
    Named(Path, Vec<Type>),
    /// Tuple type (T1, T2, T3)
    Tuple(Vec<Type>, Span),
    /// Function type (T1, T2) -> T3
    Function(Vec<Type>, Box<Type>, Span),
    /// Reference type (&T, &mut T)
    Reference(Box<Type>, bool, Span),
    /// Array type [T; N]
    Array(Box<Type>, Option<Box<Expr>>, Span),
    /// Slice type [T]
    Slice(Box<Type>, Span),
    /// Never type (!) - for functions that never return
    Never(Span),
    /// Inferred type (_)
    Infer(Span),
    /// Error type for recovery
    Error,
}

/// Path to an item (e.g., std::collections::HashMap)
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub segments: Vec<PathSegment>,
    pub span: Span,
}

/// Segment of a path
#[derive(Debug, Clone, PartialEq)]
pub struct PathSegment {
    pub ident: RcStr,
    pub args: Option<GenericArgs>,
}

/// Generic type arguments
#[derive(Debug, Clone, PartialEq)]
pub struct GenericArgs {
    pub args: Vec<GenericArg>,
    pub span: Span,
}

/// Generic argument (type, lifetime, or const)
#[derive(Debug, Clone, PartialEq)]
pub enum GenericArg {
    Type(Type),
    Lifetime(RcStr, Span),
    Const(Expr),
}

// ===== Expressions =====

/// Expressions in Vāktra
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Literal value (42, true, "string")
    Literal(Literal, Span),
    /// Variable reference (x)
    Variable(Path, Span),
    /// Field access (x.y)
    FieldAccess(Box<Expr>, RcStr, Span),
    /// Method call (x.foo(1, 2))
    MethodCall(Box<Expr>, RcStr, Vec<Type>, Vec<Expr>, Span),
    /// Function call (foo(1, 2))
    Call(Box<Expr>, Vec<Expr>, Span),
    /// Binary operation (a + b)
    Binary(Box<Expr>, BinaryOp, Box<Expr>, Span),
    /// Unary operation (!x, -y)
    Unary(UnaryOp, Box<Expr>, Span),
    /// Assignment (x = y)
    Assign(Box<Expr>, Box<Expr>, Span),
    /// Block expression { ... }
    Block(Box<Block>, Span),
    /// If expression (if x { y } else { z })
    If(Box<Expr>, Box<Block>, Option<Box<Expr>>, Span),
    /// Loop expression (loop { ... })
    Loop(Box<Block>, Option<LoopLabel>, Span),
    /// While loop (while x { ... })
    While(Box<Expr>, Box<Block>, Option<LoopLabel>, Span),
    /// For loop (for x in y { ... })
    For(Pattern, Box<Expr>, Box<Block>, Option<LoopLabel>, Span),
    /// Match expression (match x { ... })
    Match(Box<Expr>, Vec<Arm>, Span),
    /// Return expression (return x)
    Return(Option<Box<Expr>>, Span),
    /// Break expression (break 'label value)
    Break(Option<LoopLabel>, Option<Box<Expr>>, Span),
    /// Continue expression (continue 'label)
    Continue(Option<LoopLabel>, Span),
    /// Lambda expression (|x, y| x + y)
    Lambda(Vec<Param>, Box<Expr>, Span),
    /// Array literal [1, 2, 3]
    Array(Vec<Expr>, Span),
    /// Tuple literal (1, 2, 3)
    Tuple(Vec<Expr>, Span),
    /// Struct literal Point { x: 1, y: 2 }
    Struct(Path, Vec<FieldValue>, Span),
    /// Range expression (1..10, 1..=10)
    Range(Option<Box<Expr>>, Option<Box<Expr>>, RangeLimits, Span),
    /// Async block (async { ... })
    Async(Box<Block>, Span),
    /// Await expression (future.await)
    Await(Box<Expr>, Span),
    /// Try block (try { ... })
    Try(Box<Block>, Span),
    /// Error expression for recovery
    Error(Span),
}

/// Field value in a struct literal
#[derive(Debug, Clone, PartialEq)]
pub struct FieldValue {
    pub name: RcStr,
    pub value: Expr,
    pub shorthand: bool,
    pub span: Span,
}

/// Loop label for break/continue
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoopLabel {
    pub name: RcStr,
    pub span: Span,
}

/// Match arm (pattern => expression)
#[derive(Debug, Clone, PartialEq)]
pub struct Arm {
    pub pattern: Pattern,
    pub guard: Option<Guard>,
    pub body: Expr,
    pub span: Span,
}

/// Match guard (if condition)
#[derive(Debug, Clone, PartialEq)]
pub enum Guard {
    If(Box<Expr>),
    IfLet(Box<Pattern>, Box<Expr>),
}

/// Range limits (inclusive or exclusive)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeLimits {
    /// .. (exclusive)
    HalfOpen,
    /// ..= (inclusive)
    Closed,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /
    Modulo,   // %
    Equal,    // ==
    NotEqual, // !=
    Less,     // <
    LessEqual, // <=
    Greater,  // >
    GreaterEqual, // >=
    And,      // &&
    Or,       // ||
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Negate,   // -
    Not,      // !
}

/// Literal values
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Bool(bool),
    String(String),
}

// Implement Display for debugging
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Named(path, _) => write!(f, "{:?}", path),
            Type::Tuple(types, _) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", ty)?;
                }
                write!(f, ")")
            },
            Type::Function(params, ret, _) => {
                write!(f, "(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            },
            Type::Reference(ty, mutable, _) => {
                if *mutable {
                    write!(f, "&mut {}", ty)
                } else {
                    write!(f, "&{}", ty)
                }
            },
            Type::Array(ty, size, _) => {
                if let Some(size) = size {
                    write!(f, "[{}; {:?}]", ty, size)
                } else {
                    write!(f, "[{}]", ty)
                }
            },
            Type::Slice(ty, _) => write!(f, "[{}]", ty),
            Type::Never(_) => write!(f, "!"),
            Type::Infer(_) => write!(f, "_"),
            Type::Error => write!(f, "<error>"),
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Subtract => write!(f, "-"),
            BinaryOp::Multiply => write!(f, "*"),
            BinaryOp::Divide => write!(f, "/"),
            BinaryOp::Modulo => write!(f, "%"),
            BinaryOp::Equal => write!(f, "=="),
            BinaryOp::NotEqual => write!(f, "!="),
            BinaryOp::Less => write!(f, "<"),
            BinaryOp::LessEqual => write!(f, "<="),
            BinaryOp::Greater => write!(f, ">"),
            BinaryOp::GreaterEqual => write!(f, ">="),
            BinaryOp::And => write!(f, "&&"),
            BinaryOp::Or => write!(f, "||"),
        }
    }
}
