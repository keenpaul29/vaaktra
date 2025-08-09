//! Semantic Analysis for Vāktra (वाक्त्र)
//! 
//! Provides type checking, symbol resolution, and semantic validation
//! inspired by Vedic principles of knowledge (ज्ञान) and understanding (बोध).

pub mod analyzer;
pub mod type_checker;
pub mod symbol_table;
pub mod error;

use std::collections::HashMap;
use vaaktra_parser::ast::{Program, Item, Type, Expr};
use thiserror::Error;

/// Semantic analysis errors
#[derive(Debug, Error)]
pub enum SemanticError {
    #[error("Type error: {0}")]
    TypeError(String),
    
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),
    
    #[error("Duplicate symbol: {0}")]
    DuplicateSymbol(String),
    
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub type SemanticResult<T> = Result<T, SemanticError>;

/// Main semantic analyzer for Vāktra programs
pub struct VaaktraSemanticAnalyzer {
    /// Symbol table for tracking declarations
    symbol_table: symbol_table::SymbolTable,
    
    /// Type checker for validating types
    type_checker: type_checker::TypeChecker,
    
    /// Current analysis context
    context: AnalysisContext,
}

/// Analysis context for tracking current scope and state
#[derive(Debug, Default)]
pub struct AnalysisContext {
    /// Current function being analyzed
    current_function: Option<String>,
    
    /// Current class being analyzed
    current_class: Option<String>,
    
    /// Loop nesting level
    loop_depth: usize,
    
    /// Return type of current function
    expected_return_type: Option<Type>,
}

impl VaaktraSemanticAnalyzer {
    /// Create a new semantic analyzer
    pub fn new() -> Self {
        VaaktraSemanticAnalyzer {
            symbol_table: symbol_table::SymbolTable::new(),
            type_checker: type_checker::TypeChecker::new(),
            context: AnalysisContext::default(),
        }
    }
    
    /// Analyze a complete program
    pub fn analyze_program(&mut self, program: &Program) -> SemanticResult<()> {
        log::info!("Starting semantic analysis of Vāktra program");
        
        // First pass: collect all declarations
        for item in &program.items {
            self.collect_declarations(item)?;
        }
        
        // Second pass: analyze implementations
        for item in &program.items {
            self.analyze_item(item)?;
        }
        
        log::info!("Semantic analysis completed successfully");
        Ok(())
    }
    
    /// Collect declarations in first pass
    fn collect_declarations(&mut self, item: &Item) -> SemanticResult<()> {
        match item {
            Item::Mantra(mantra) => {
                self.symbol_table.declare_function(
                    &mantra.name,
                    &mantra.params,
                    &mantra.return_type,
                )?;
            }
            Item::Dharma(dharma) => {
                self.symbol_table.declare_class(&dharma.name, &dharma.fields)?;
            }
            Item::Sutra(sutra) => {
                self.symbol_table.declare_variable(&sutra.name, &sutra.var_type)?;
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Analyze an item in second pass
    fn analyze_item(&mut self, item: &Item) -> SemanticResult<()> {
        match item {
            Item::Mantra(mantra) => {
                self.context.current_function = Some(mantra.name.clone());
                self.context.expected_return_type = Some(mantra.return_type.clone());
                
                // Analyze function body
                if let Some(body) = &mantra.body {
                    for statement in body {
                        self.analyze_statement(statement)?;
                    }
                }
                
                self.context.current_function = None;
                self.context.expected_return_type = None;
            }
            Item::Dharma(dharma) => {
                self.context.current_class = Some(dharma.name.clone());
                
                // Analyze class methods
                for method in &dharma.methods {
                    self.analyze_item(&Item::Mantra(method.clone()))?;
                }
                
                self.context.current_class = None;
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Analyze a statement
    fn analyze_statement(&mut self, statement: &vaaktra_parser::ast::Statement) -> SemanticResult<()> {
        // Implementation would analyze different statement types
        Ok(())
    }
    
    /// Get the symbol table
    pub fn symbol_table(&self) -> &symbol_table::SymbolTable {
        &self.symbol_table
    }
    
    /// Get the type checker
    pub fn type_checker(&self) -> &type_checker::TypeChecker {
        &self.type_checker
    }
}

impl Default for VaaktraSemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
