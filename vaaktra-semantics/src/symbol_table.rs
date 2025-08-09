//! Symbol Table for Vāktra (वाक्त्र)
//! 
//! Manages symbol resolution and scoping inspired by Vedic concepts
//! of knowledge hierarchy (ज्ञान पदानुक्रम).

use std::collections::HashMap;
use vaaktra_parser::ast::{Type, Param, FieldDef};
use crate::{SemanticError, SemanticResult};

/// Symbol information
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub scope_level: usize,
    pub is_mutable: bool,
    pub location: Option<(usize, usize)>, // line, column
}

/// Types of symbols
#[derive(Debug, Clone)]
pub enum SymbolType {
    /// Variable (सूत्र)
    Variable(Type),
    
    /// Function (मन्त्र)
    Function {
        params: Vec<Param>,
        return_type: Type,
    },
    
    /// Class (धर्म)
    Class {
        fields: Vec<FieldDef>,
        methods: Vec<String>,
    },
    
    /// Module (यन्त्र)
    Module {
        exports: Vec<String>,
    },
    
    /// Type alias
    TypeAlias(Type),
}

/// Symbol table with hierarchical scoping
pub struct SymbolTable {
    /// Scopes stack (outermost to innermost)
    scopes: Vec<HashMap<String, Symbol>>,
    
    /// Current scope level
    current_level: usize,
    
    /// Global symbols cache for fast lookup
    global_cache: HashMap<String, Symbol>,
}

impl SymbolTable {
    /// Create a new symbol table
    pub fn new() -> Self {
        let mut table = SymbolTable {
            scopes: Vec::new(),
            current_level: 0,
            global_cache: HashMap::new(),
        };
        
        // Create global scope
        table.push_scope();
        
        // Add built-in types and functions
        table.add_builtins();
        
        table
    }
    
    /// Add built-in symbols
    fn add_builtins(&mut self) {
        // Built-in types
        self.declare_builtin_type("सङ्ख्या", "Number type");
        self.declare_builtin_type("सत्यासत्य", "Boolean type");
        self.declare_builtin_type("शब्द", "String type");
        self.declare_builtin_type("सूची", "List type");
        self.declare_builtin_type("निधान", "Map type");
        self.declare_builtin_type("शून्य", "Void type");
        
        // Built-in functions
        self.declare_builtin_function("प्रिंट", "Print function", vec![], Type::Named {
            name: "शून्य".into(),
            generics: None,
        });
    }
    
    /// Declare a built-in type
    fn declare_builtin_type(&mut self, name: &str, _description: &str) {
        let symbol = Symbol {
            name: name.to_string(),
            symbol_type: SymbolType::TypeAlias(Type::Named {
                name: name.into(),
                generics: None,
            }),
            scope_level: 0,
            is_mutable: false,
            location: None,
        };
        
        self.scopes[0].insert(name.to_string(), symbol.clone());
        self.global_cache.insert(name.to_string(), symbol);
    }
    
    /// Declare a built-in function
    fn declare_builtin_function(&mut self, name: &str, _description: &str, params: Vec<Param>, return_type: Type) {
        let symbol = Symbol {
            name: name.to_string(),
            symbol_type: SymbolType::Function {
                params,
                return_type,
            },
            scope_level: 0,
            is_mutable: false,
            location: None,
        };
        
        self.scopes[0].insert(name.to_string(), symbol.clone());
        self.global_cache.insert(name.to_string(), symbol);
    }
    
    /// Push a new scope
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.current_level += 1;
    }
    
    /// Pop the current scope
    pub fn pop_scope(&mut self) -> SemanticResult<()> {
        if self.scopes.len() <= 1 {
            return Err(SemanticError::InvalidOperation("Cannot pop global scope".to_string()));
        }
        
        self.scopes.pop();
        self.current_level -= 1;
        Ok(())
    }
    
    /// Declare a variable
    pub fn declare_variable(&mut self, name: &str, var_type: &Type) -> SemanticResult<()> {
        if self.current_scope_contains(name) {
            return Err(SemanticError::DuplicateSymbol(name.to_string()));
        }
        
        let symbol = Symbol {
            name: name.to_string(),
            symbol_type: SymbolType::Variable(var_type.clone()),
            scope_level: self.current_level,
            is_mutable: true,
            location: None,
        };
        
        self.current_scope_mut().insert(name.to_string(), symbol);
        Ok(())
    }
    
    /// Declare a function
    pub fn declare_function(&mut self, name: &str, params: &[Param], return_type: &Type) -> SemanticResult<()> {
        if self.current_scope_contains(name) {
            return Err(SemanticError::DuplicateSymbol(name.to_string()));
        }
        
        let symbol = Symbol {
            name: name.to_string(),
            symbol_type: SymbolType::Function {
                params: params.to_vec(),
                return_type: return_type.clone(),
            },
            scope_level: self.current_level,
            is_mutable: false,
            location: None,
        };
        
        self.current_scope_mut().insert(name.to_string(), symbol.clone());
        
        // Cache global functions
        if self.current_level == 1 {
            self.global_cache.insert(name.to_string(), symbol);
        }
        
        Ok(())
    }
    
    /// Declare a class
    pub fn declare_class(&mut self, name: &str, fields: &[FieldDef]) -> SemanticResult<()> {
        if self.current_scope_contains(name) {
            return Err(SemanticError::DuplicateSymbol(name.to_string()));
        }
        
        let symbol = Symbol {
            name: name.to_string(),
            symbol_type: SymbolType::Class {
                fields: fields.to_vec(),
                methods: Vec::new(),
            },
            scope_level: self.current_level,
            is_mutable: false,
            location: None,
        };
        
        self.current_scope_mut().insert(name.to_string(), symbol.clone());
        
        // Cache global classes
        if self.current_level == 1 {
            self.global_cache.insert(name.to_string(), symbol);
        }
        
        Ok(())
    }
    
    /// Look up a symbol
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        // Check global cache first for performance
        if let Some(symbol) = self.global_cache.get(name) {
            return Some(symbol);
        }
        
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        
        None
    }
    
    /// Check if symbol exists in current scope
    fn current_scope_contains(&self, name: &str) -> bool {
        self.current_scope().contains_key(name)
    }
    
    /// Get current scope
    fn current_scope(&self) -> &HashMap<String, Symbol> {
        self.scopes.last().unwrap()
    }
    
    /// Get mutable reference to current scope
    fn current_scope_mut(&mut self) -> &mut HashMap<String, Symbol> {
        self.scopes.last_mut().unwrap()
    }
    
    /// Get all symbols in current scope
    pub fn current_scope_symbols(&self) -> Vec<&Symbol> {
        self.current_scope().values().collect()
    }
    
    /// Get all symbols across all scopes
    pub fn all_symbols(&self) -> Vec<&Symbol> {
        let mut symbols = Vec::new();
        for scope in &self.scopes {
            symbols.extend(scope.values());
        }
        symbols
    }
    
    /// Check if we're in global scope
    pub fn is_global_scope(&self) -> bool {
        self.current_level == 1
    }
    
    /// Get current scope level
    pub fn current_scope_level(&self) -> usize {
        self.current_level
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
