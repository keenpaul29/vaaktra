//! Error handling for Vāktra (वाक्त्र) semantic analysis
//! 
//! Provides comprehensive error reporting with Sanskrit-inspired messages
//! for better developer experience.

use std::fmt;
use vaaktra_parser::ast::Span;

/// Detailed semantic error with location information
#[derive(Debug, Clone)]
pub struct DetailedSemanticError {
    pub error_type: SemanticErrorType,
    pub message: String,
    pub span: Option<Span>,
    pub suggestions: Vec<String>,
    pub related_errors: Vec<Box<DetailedSemanticError>>,
}

/// Types of semantic errors
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticErrorType {
    /// Type mismatch
    TypeMismatch,
    
    /// Symbol not found
    SymbolNotFound,
    
    /// Duplicate symbol
    DuplicateSymbol,
    
    /// Invalid operation
    InvalidOperation,
    
    /// Missing return statement
    MissingReturn,
    
    /// Unreachable code
    UnreachableCode,
    
    /// Invalid assignment
    InvalidAssignment,
    
    /// Circular dependency
    CircularDependency,
}

impl DetailedSemanticError {
    /// Create a new detailed error
    pub fn new(error_type: SemanticErrorType, message: String) -> Self {
        DetailedSemanticError {
            error_type,
            message,
            span: None,
            suggestions: Vec::new(),
            related_errors: Vec::new(),
        }
    }
    
    /// Add location information
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
    
    /// Add suggestion
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestions.push(suggestion);
        self
    }
    
    /// Add related error
    pub fn with_related_error(mut self, error: DetailedSemanticError) -> Self {
        self.related_errors.push(Box::new(error));
        self
    }
    
    /// Get Sanskrit-inspired error message
    pub fn sanskrit_message(&self) -> String {
        match self.error_type {
            SemanticErrorType::TypeMismatch => {
                format!("प्रकार असंगति (Type Mismatch): {}", self.message)
            }
            SemanticErrorType::SymbolNotFound => {
                format!("प्रतीक अनुपस्थित (Symbol Not Found): {}", self.message)
            }
            SemanticErrorType::DuplicateSymbol => {
                format!("द्विगुण प्रतीक (Duplicate Symbol): {}", self.message)
            }
            SemanticErrorType::InvalidOperation => {
                format!("अवैध क्रिया (Invalid Operation): {}", self.message)
            }
            SemanticErrorType::MissingReturn => {
                format!("प्रत्यावर्तन अनुपस्थित (Missing Return): {}", self.message)
            }
            SemanticErrorType::UnreachableCode => {
                format!("अगम्य कोड (Unreachable Code): {}", self.message)
            }
            SemanticErrorType::InvalidAssignment => {
                format!("अवैध निर्देशन (Invalid Assignment): {}", self.message)
            }
            SemanticErrorType::CircularDependency => {
                format!("चक्रीय निर्भरता (Circular Dependency): {}", self.message)
            }
        }
    }
}

impl fmt::Display for DetailedSemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.sanskrit_message())?;
        
        if let Some(span) = &self.span {
            writeln!(f, "  स्थान (Location): {}:{}", span.start, span.end)?;
        }
        
        if !self.suggestions.is_empty() {
            writeln!(f, "  सुझाव (Suggestions):")?;
            for suggestion in &self.suggestions {
                writeln!(f, "    - {}", suggestion)?;
            }
        }
        
        if !self.related_errors.is_empty() {
            writeln!(f, "  संबंधित त्रुटियाँ (Related Errors):")?;
            for error in &self.related_errors {
                writeln!(f, "    {}", error)?;
            }
        }
        
        Ok(())
    }
}

impl std::error::Error for DetailedSemanticError {}

/// Error collector for gathering multiple errors
pub struct ErrorCollector {
    errors: Vec<DetailedSemanticError>,
    warnings: Vec<DetailedSemanticError>,
}

impl ErrorCollector {
    /// Create a new error collector
    pub fn new() -> Self {
        ErrorCollector {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    /// Add an error
    pub fn add_error(&mut self, error: DetailedSemanticError) {
        self.errors.push(error);
    }
    
    /// Add a warning
    pub fn add_warning(&mut self, warning: DetailedSemanticError) {
        self.warnings.push(warning);
    }
    
    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    /// Get all errors
    pub fn errors(&self) -> &[DetailedSemanticError] {
        &self.errors
    }
    
    /// Get all warnings
    pub fn warnings(&self) -> &[DetailedSemanticError] {
        &self.warnings
    }
    
    /// Clear all errors and warnings
    pub fn clear(&mut self) {
        self.errors.clear();
        self.warnings.clear();
    }
    
    /// Print all errors and warnings
    pub fn print_all(&self) {
        for error in &self.errors {
            eprintln!("त्रुटि (Error): {}", error);
        }
        
        for warning in &self.warnings {
            eprintln!("चेतावनी (Warning): {}", warning);
        }
    }
}

impl Default for ErrorCollector {
    fn default() -> Self {
        Self::new()
    }
}
