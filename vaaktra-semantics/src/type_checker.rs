//! Type Checker for Vāktra (वाक्त्र)
//! 
//! Provides static type checking with advanced inference capabilities
//! inspired by Vedic principles of logical reasoning (तर्क).

use std::collections::HashMap;
use vaaktra_parser::ast::{Type, Expr, BinaryOp, UnaryOp};
use crate::{SemanticError, SemanticResult};

/// Type checker with advanced inference
pub struct TypeChecker {
    /// Type inference cache
    inference_cache: HashMap<String, Type>,
    
    /// Generic type constraints
    constraints: HashMap<String, Vec<TypeConstraint>>,
    
    /// Current type context
    context: TypeContext,
}

/// Type constraint for generic types
#[derive(Debug, Clone)]
pub struct TypeConstraint {
    pub type_param: String,
    pub constraint_type: Type,
    pub constraint_kind: ConstraintKind,
}

/// Kinds of type constraints
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintKind {
    /// Must implement trait
    Implements,
    
    /// Must be subtype of
    SubtypeOf,
    
    /// Must be same as
    Equals,
    
    /// Must be numeric
    Numeric,
    
    /// Must be comparable
    Comparable,
}

/// Type checking context
#[derive(Debug, Default)]
pub struct TypeContext {
    /// Expected type (for inference)
    expected_type: Option<Type>,
    
    /// Current function return type
    return_type: Option<Type>,
    
    /// Generic type parameters in scope
    type_params: HashMap<String, Type>,
}

impl TypeChecker {
    /// Create a new type checker
    pub fn new() -> Self {
        TypeChecker {
            inference_cache: HashMap::new(),
            constraints: HashMap::new(),
            context: TypeContext::default(),
        }
    }
    
    /// Check if two types are compatible
    pub fn are_compatible(&self, left: &Type, right: &Type) -> bool {
        match (left, right) {
            // Same named types
            (Type::Named { name: n1, .. }, Type::Named { name: n2, .. }) => n1 == n2,
            
            // Function types
            (Type::Function { params: p1, return_type: r1, .. }, 
             Type::Function { params: p2, return_type: r2, .. }) => {
                p1.len() == p2.len() &&
                p1.iter().zip(p2.iter()).all(|(t1, t2)| self.are_compatible(t1, t2)) &&
                self.are_compatible(r1, r2)
            }
            
            // Array types
            (Type::Array { element_type: e1, .. }, Type::Array { element_type: e2, .. }) => {
                self.are_compatible(e1, e2)
            }
            
            // Tuple types
            (Type::Tuple { elements: e1, .. }, Type::Tuple { elements: e2, .. }) => {
                e1.len() == e2.len() &&
                e1.iter().zip(e2.iter()).all(|(t1, t2)| self.are_compatible(t1, t2))
            }
            
            // Reference types
            (Type::Reference { target: t1, .. }, Type::Reference { target: t2, .. }) => {
                self.are_compatible(t1, t2)
            }
            
            // Generic types (simplified)
            (Type::Generic { name: n1, .. }, Type::Generic { name: n2, .. }) => n1 == n2,
            
            _ => false,
        }
    }
    
    /// Infer the type of an expression
    pub fn infer_expression_type(&mut self, expr: &Expr) -> SemanticResult<Type> {
        match expr {
            Expr::Literal { value, .. } => self.infer_literal_type(value),
            
            Expr::Identifier { name, .. } => {
                // Look up in inference cache or symbol table
                if let Some(cached_type) = self.inference_cache.get(name) {
                    Ok(cached_type.clone())
                } else {
                    Err(SemanticError::SymbolNotFound(name.clone()))
                }
            }
            
            Expr::Binary { left, op, right, .. } => {
                let left_type = self.infer_expression_type(left)?;
                let right_type = self.infer_expression_type(right)?;
                self.infer_binary_op_type(&left_type, op, &right_type)
            }
            
            Expr::Unary { op, operand, .. } => {
                let operand_type = self.infer_expression_type(operand)?;
                self.infer_unary_op_type(op, &operand_type)
            }
            
            Expr::Call { function, args, .. } => {
                self.infer_call_type(function, args)
            }
            
            Expr::Array { elements, .. } => {
                if elements.is_empty() {
                    // Empty array - use expected type or default to generic
                    Ok(Type::Array {
                        element_type: Box::new(Type::Generic {
                            name: "T".into(),
                            bounds: vec![],
                        }),
                        size: Some(0),
                    })
                } else {
                    let element_type = self.infer_expression_type(&elements[0])?;
                    
                    // Check all elements have same type
                    for element in &elements[1..] {
                        let elem_type = self.infer_expression_type(element)?;
                        if !self.are_compatible(&element_type, &elem_type) {
                            return Err(SemanticError::TypeMismatch {
                                expected: self.type_to_string(&element_type),
                                found: self.type_to_string(&elem_type),
                            });
                        }
                    }
                    
                    Ok(Type::Array {
                        element_type: Box::new(element_type),
                        size: Some(elements.len()),
                    })
                }
            }
            
            Expr::Struct { name, fields, .. } => {
                Ok(Type::Named {
                    name: name.clone(),
                    generics: None,
                })
            }
            
            _ => Err(SemanticError::TypeError("Unsupported expression for type inference".to_string())),
        }
    }
    
    /// Infer type of a literal
    fn infer_literal_type(&self, literal: &str) -> SemanticResult<Type> {
        // Try to parse as different types
        if literal.parse::<i64>().is_ok() || literal.chars().all(|c| "०१२३४५६७८९".contains(c)) {
            Ok(Type::Named {
                name: "सङ्ख्या".into(),
                generics: None,
            })
        } else if literal == "सत्य" || literal == "असत्य" {
            Ok(Type::Named {
                name: "सत्यासत्य".into(),
                generics: None,
            })
        } else if literal.starts_with('"') && literal.ends_with('"') {
            Ok(Type::Named {
                name: "शब्द".into(),
                generics: None,
            })
        } else {
            Err(SemanticError::TypeError(format!("Unknown literal type: {}", literal)))
        }
    }
    
    /// Infer type of binary operation
    fn infer_binary_op_type(&self, left: &Type, op: &BinaryOp, right: &Type) -> SemanticResult<Type> {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                // Arithmetic operations require numeric types
                if self.is_numeric_type(left) && self.is_numeric_type(right) {
                    if self.are_compatible(left, right) {
                        Ok(left.clone())
                    } else {
                        // Try to find common numeric type
                        self.find_common_numeric_type(left, right)
                    }
                } else {
                    Err(SemanticError::TypeError("Arithmetic operations require numeric types".to_string()))
                }
            }
            
            BinaryOp::Eq | BinaryOp::Ne => {
                // Equality operations return boolean
                if self.are_compatible(left, right) {
                    Ok(Type::Named {
                        name: "सत्यासत्य".into(),
                        generics: None,
                    })
                } else {
                    Err(SemanticError::TypeMismatch {
                        expected: self.type_to_string(left),
                        found: self.type_to_string(right),
                    })
                }
            }
            
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                // Comparison operations require comparable types
                if self.is_comparable_type(left) && self.are_compatible(left, right) {
                    Ok(Type::Named {
                        name: "सत्यासत्य".into(),
                        generics: None,
                    })
                } else {
                    Err(SemanticError::TypeError("Comparison operations require comparable types".to_string()))
                }
            }
            
            BinaryOp::And | BinaryOp::Or => {
                // Logical operations require boolean types
                if self.is_boolean_type(left) && self.is_boolean_type(right) {
                    Ok(Type::Named {
                        name: "सत्यासत्य".into(),
                        generics: None,
                    })
                } else {
                    Err(SemanticError::TypeError("Logical operations require boolean types".to_string()))
                }
            }
        }
    }
    
    /// Infer type of unary operation
    fn infer_unary_op_type(&self, op: &UnaryOp, operand: &Type) -> SemanticResult<Type> {
        match op {
            UnaryOp::Neg => {
                if self.is_numeric_type(operand) {
                    Ok(operand.clone())
                } else {
                    Err(SemanticError::TypeError("Negation requires numeric type".to_string()))
                }
            }
            
            UnaryOp::Not => {
                if self.is_boolean_type(operand) {
                    Ok(operand.clone())
                } else {
                    Err(SemanticError::TypeError("Logical not requires boolean type".to_string()))
                }
            }
        }
    }
    
    /// Infer type of function call
    fn infer_call_type(&mut self, _function: &str, _args: &[Expr]) -> SemanticResult<Type> {
        // Simplified - would need to look up function signature
        Ok(Type::Named {
            name: "शून्य".into(),
            generics: None,
        })
    }
    
    /// Check if type is numeric
    fn is_numeric_type(&self, type_: &Type) -> bool {
        match type_ {
            Type::Named { name, .. } => {
                matches!(name.as_str(), "सङ्ख्या" | "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64")
            }
            _ => false,
        }
    }
    
    /// Check if type is boolean
    fn is_boolean_type(&self, type_: &Type) -> bool {
        match type_ {
            Type::Named { name, .. } => name == "सत्यासत्य" || name == "bool",
            _ => false,
        }
    }
    
    /// Check if type is comparable
    fn is_comparable_type(&self, type_: &Type) -> bool {
        self.is_numeric_type(type_) || self.is_boolean_type(type_) || 
        match type_ {
            Type::Named { name, .. } => name == "शब्द" || name == "String",
            _ => false,
        }
    }
    
    /// Find common numeric type for two types
    fn find_common_numeric_type(&self, left: &Type, right: &Type) -> SemanticResult<Type> {
        // Simplified type promotion rules
        match (left, right) {
            (Type::Named { name: n1, .. }, Type::Named { name: n2, .. }) => {
                if n1 == "सङ्ख्या" || n2 == "सङ्ख्या" {
                    Ok(Type::Named {
                        name: "सङ्ख्या".into(),
                        generics: None,
                    })
                } else {
                    Err(SemanticError::TypeMismatch {
                        expected: self.type_to_string(left),
                        found: self.type_to_string(right),
                    })
                }
            }
            _ => Err(SemanticError::TypeError("Cannot find common numeric type".to_string())),
        }
    }
    
    /// Convert type to string representation
    fn type_to_string(&self, type_: &Type) -> String {
        match type_ {
            Type::Named(path, _) => path.segments.last().map(|s| s.name.as_str()).unwrap_or("unknown").to_string(),
            Type::Function(..) => "function".to_string(),
            Type::Array(element_type, _, _) => format!("[{}]", self.type_to_string(element_type)),
            Type::Tuple(elements, _) => {
                let elem_strs: Vec<String> = elements.iter().map(|t| self.type_to_string(t)).collect();
                format!("({})", elem_strs.join(", "))
            }
            Type::Reference { target, .. } => format!("&{}", self.type_to_string(target)),
            Type::Generic { name, .. } => name.clone(),
            Type::Slice { element_type, .. } => format!("[{}]", self.type_to_string(element_type)),
        }
    }
    
    /// Set expected type for inference
    pub fn set_expected_type(&mut self, expected: Option<Type>) {
        self.context.expected_type = expected;
    }
    
    /// Add type to inference cache
    pub fn cache_type(&mut self, name: String, type_: Type) {
        self.inference_cache.insert(name, type_);
    }
    
    /// Clear inference cache
    pub fn clear_cache(&mut self) {
        self.inference_cache.clear();
    }
    
    /// Add type constraint
    pub fn add_constraint(&mut self, type_param: String, constraint: TypeConstraint) {
        self.constraints.entry(type_param).or_insert_with(Vec::new).push(constraint);
    }
    
    /// Check if constraints are satisfied
    pub fn check_constraints(&self) -> SemanticResult<()> {
        for (type_param, constraints) in &self.constraints {
            for constraint in constraints {
                // Simplified constraint checking
                log::debug!("Checking constraint for {}: {:?}", type_param, constraint);
            }
        }
        Ok(())
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
