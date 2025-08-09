//! JIT Compiler for Vāktra (वाक्त्र)
//! 
//! This module implements Just-In-Time compilation using LLVM.
//! Transforms AST into optimized machine code for maximum performance.

use crate::{JitError, JitResult};
use vaaktra_parser::ast::*;
use std::collections::HashMap;

#[cfg(all(feature = "llvm", not(vaaktra_no_jit)))]
use inkwell::{
    context::Context,
    module::Module,
    builder::Builder,
    execution_engine::{ExecutionEngine, JitFunction},
    values::{BasicValueEnum, BasicValue, FunctionValue, PointerValue},
    types::{BasicTypeEnum, BasicType, FunctionType},
    OptimizationLevel,
};
#[cfg(feature = "llvm")]
use inkwell::types::{BasicTypeEnum, FunctionType};
#[cfg(feature = "llvm")]
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
#[cfg(feature = "llvm")]
use inkwell::{OptimizationLevel as LLVMOptLevel, AddressSpace};

use std::collections::HashMap;
use thiserror::Error;

use vaaktra_parser::ast::{Program, Item, MantraDef, DharmaDef, Expr, Statement, Type};
use crate::{OptimizationLevel, VaaktraType, JitResult, JitError};

/// Compilation errors
#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("LLVM error: {0}")]
    LLVMError(String),
    
    #[error("Type error: {0}")]
    TypeError(String),
    
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),
    
    #[error("Code generation failed: {0}")]
    CodeGenFailed(String),
}

/// LLVM-based compiler for maximum performance
pub struct VaaktraCompiler<'ctx> {
    /// LLVM context
    context: &'ctx Context,
    
    /// LLVM module
    module: Module<'ctx>,
    
    /// LLVM builder
    builder: Builder<'ctx>,
    
    /// Execution engine for JIT
    execution_engine: ExecutionEngine<'ctx>,
    
    /// Symbol table for functions and variables
    symbols: HashMap<String, BasicValueEnum<'ctx>>,
    
    /// Function table
    functions: HashMap<String, FunctionValue<'ctx>>,
    
    /// Current optimization level
    optimization_level: OptimizationLevel,
}

impl<'ctx> VaaktraCompiler<'ctx> {
    /// Create a new compiler with LLVM backend
    pub fn new(context: &'ctx Context, module_name: &str) -> JitResult<Self> {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        
        let execution_engine = module
            .create_jit_execution_engine(LLVMOptLevel::Aggressive)
            .map_err(|e| JitError::CompilationFailed(format!("Failed to create execution engine: {}", e)))?;
        
        Ok(VaaktraCompiler {
            context,
            module,
            builder,
            execution_engine,
            symbols: HashMap::new(),
            functions: HashMap::new(),
            optimization_level: OptimizationLevel::Sattva,
        })
    }
    
    /// Compile a complete Vāktra program
    pub fn compile_program(&mut self, program: &Program) -> JitResult<()> {
        log::info!("Starting LLVM compilation of Vāktra program");
        
        // First pass: declare all functions and types
        for item in &program.items {
            match item {
                Item::Mantra(mantra) => {
                    self.declare_function(mantra)?;
                }
                Item::Dharma(dharma) => {
                    self.declare_struct(dharma)?;
                }
                _ => {}
            }
        }
        
        // Second pass: compile function bodies
        for item in &program.items {
            match item {
                Item::Mantra(mantra) => {
                    self.compile_function(mantra)?;
                }
                _ => {}
            }
        }
        
        // Verify the module
        if let Err(errors) = self.module.verify() {
            return Err(JitError::CompilationFailed(format!("Module verification failed: {}", errors)));
        }
        
        log::info!("LLVM compilation completed successfully");
        Ok(())
    }
    
    /// Declare a function signature
    fn declare_function(&mut self, mantra: &MantraDef) -> JitResult<FunctionValue<'ctx>> {
        log::debug!("Declaring function: {}", mantra.name);
        
        // Convert parameter types
        let param_types: Vec<BasicTypeEnum> = mantra.params.iter()
            .map(|param| self.convert_type(&param.ty))
            .collect::<Result<Vec<_>, _>>()?;
        
        // Convert return type
        let return_type = self.convert_type(&mantra.return_type)?;
        
        // Create function type
        let fn_type = return_type.fn_type(&param_types, false);
        
        // Add function to module
        let function = self.module.add_function(&mantra.name, fn_type, None);
        
        // Set parameter names
        for (i, param) in mantra.params.iter().enumerate() {
            if let Some(param_value) = function.get_nth_param(i as u32) {
                param_value.set_name(&param.name);
            }
        }
        
        self.functions.insert(mantra.name.clone(), function);
        Ok(function)
    }
    
    /// Declare a struct type
    fn declare_struct(&mut self, dharma: &DharmaDef) -> JitResult<()> {
        log::debug!("Declaring struct: {}", dharma.name);
        
        // Convert field types
        let field_types: Vec<BasicTypeEnum> = dharma.fields.iter()
            .map(|field| self.convert_type(&field.ty))
            .collect::<Result<Vec<_>, _>>()?;
        
        // Create struct type
        let struct_type = self.context.struct_type(&field_types, false);
        
        // TODO: Store struct type for later use
        
        Ok(())
    }
    
    /// Compile a function body
    fn compile_function(&mut self, mantra: &MantraDef) -> JitResult<()> {
        log::debug!("Compiling function body: {}", mantra.name);
        
        let function = self.functions.get(&mantra.name)
            .ok_or_else(|| JitError::CompilationFailed(format!("Function {} not declared", mantra.name)))?
            .clone();
        
        // Create entry block
        let entry_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_block);
        
        // Create symbol table for parameters
        let mut local_symbols = HashMap::new();
        for (i, param) in mantra.params.iter().enumerate() {
            if let Some(param_value) = function.get_nth_param(i as u32) {
                local_symbols.insert(param.name.clone(), param_value);
            }
        }
        
        // Compile function body
        if let Some(ref body) = mantra.body {
            let return_value = self.compile_block(body, &mut local_symbols)?;
            
            // Add return instruction
            if let Some(ret_val) = return_value {
                self.builder.build_return(Some(&ret_val));
            } else {
                // Void return
                self.builder.build_return(None);
            }
        } else {
            // Empty function, return default value
            let return_type = self.convert_type(&mantra.return_type)?;
            let default_value = self.get_default_value(return_type);
            self.builder.build_return(Some(&default_value));
        }
        
        Ok(())
    }
    
    /// Compile a block of statements
    fn compile_block(&mut self, statements: &[Statement], local_symbols: &mut HashMap<String, BasicValueEnum<'ctx>>) -> JitResult<Option<BasicValueEnum<'ctx>>> {
        let mut last_value = None;
        
        for statement in statements {
            match statement {
                Statement::Expression(expr) => {
                    last_value = Some(self.compile_expression(expr, local_symbols)?);
                }
                Statement::Let { name, value, .. } => {
                    let val = self.compile_expression(value, local_symbols)?;
                    local_symbols.insert(name.clone(), val);
                }
                _ => {
                    // Handle other statement types
                    log::warn!("Unhandled statement type in compilation");
                }
            }
        }
        
        Ok(last_value)
    }
    
    /// Compile an expression
    fn compile_expression(&mut self, expr: &Expr, local_symbols: &HashMap<String, BasicValueEnum<'ctx>>) -> JitResult<BasicValueEnum<'ctx>> {
        match expr {
            Expr::Literal(literal, _span) => {
                match literal {
                    vaaktra_parser::ast::Literal::Int(n) => Ok(self.context.i64_type().const_int(*n as u64, false).into()),
                    vaaktra_parser::ast::Literal::Bool(b) => Ok(self.context.bool_type().const_int(*b as u64, false).into()),
                    vaaktra_parser::ast::Literal::String(s) => {
                        let string_val = self.context.const_string(s.as_bytes(), false);
                        Ok(string_val.into())
                    }
                }
            }
            Expr::Variable(path, _span) => {
                let name = path.segments.last().unwrap().as_str();
                local_symbols.get(name)
                    .or_else(|| self.symbols.get(name))
                    .copied()
                    .ok_or_else(|| JitError::CompilationFailed(format!("Symbol {} not found", name)))
            }
            Expr::Binary(left, op, right, _span) => {
                let left_val = self.compile_expression(left, local_symbols)?;
                let right_val = self.compile_expression(right, local_symbols)?;
                self.compile_binary_op(left_val, &format!("{:?}", op), right_val)
            }
            Expr::Call(function, args, _span) => {
                // For now, assume function is a variable reference
                if let Expr::Variable(path, _) = function.as_ref() {
                    let function_name = path.segments.last().unwrap().as_str();
                    self.compile_function_call(function_name, args, local_symbols)
                } else {
                    Err(JitError::CompilationFailed("Complex function calls not yet supported".to_string()))
                }
            }
            _ => {
                Err(JitError::CompilationFailed("Unsupported expression type".to_string()))
            }
        }
    }
    
    /// Compile a literal value
    fn compile_literal(&self, literal: &str) -> JitResult<BasicValueEnum<'ctx>> {
        // Try to parse as different types
        if let Ok(int_val) = literal.parse::<i64>() {
            Ok(self.context.i64_type().const_int(int_val as u64, true).into())
        } else if literal == "सत्य" {
            Ok(self.context.bool_type().const_int(1, false).into())
        } else if literal == "असत्य" {
            Ok(self.context.bool_type().const_int(0, false).into())
        } else if literal.starts_with('"') && literal.ends_with('"') {
            // String literal
            let string_content = &literal[1..literal.len()-1];
            let string_val = self.context.const_string(string_content.as_bytes(), false);
            Ok(string_val.into())
        } else {
            Err(JitError::CompilationFailed(format!("Unsupported literal: {}", literal)))
        }
    }
    
    /// Compile a binary operation
    fn compile_binary_op(&mut self, left: BasicValueEnum<'ctx>, op: &str, right: BasicValueEnum<'ctx>) -> JitResult<BasicValueEnum<'ctx>> {
        match op {
            "धन" | "+" => {
                if let (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) = (left, right) {
                    Ok(self.builder.build_int_add(l, r, "add").into())
                } else {
                    Err(JitError::CompilationFailed("Type mismatch in addition".to_string()))
                }
            }
            "ऋण" | "-" => {
                if let (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) = (left, right) {
                    Ok(self.builder.build_int_sub(l, r, "sub").into())
                } else {
                    Err(JitError::CompilationFailed("Type mismatch in subtraction".to_string()))
                }
            }
            "गुण" | "*" => {
                if let (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) = (left, right) {
                    Ok(self.builder.build_int_mul(l, r, "mul").into())
                } else {
                    Err(JitError::CompilationFailed("Type mismatch in multiplication".to_string()))
                }
            }
            "भाग" | "/" => {
                if let (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) = (left, right) {
                    Ok(self.builder.build_int_signed_div(l, r, "div").into())
                } else {
                    Err(JitError::CompilationFailed("Type mismatch in division".to_string()))
                }
            }
            "समान" | "==" => {
                if let (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) = (left, right) {
                    Ok(self.builder.build_int_compare(inkwell::IntPredicate::EQ, l, r, "eq").into())
                } else {
                    Err(JitError::CompilationFailed("Type mismatch in equality".to_string()))
                }
            }
            "लघुत्तर" | "<" => {
                if let (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) = (left, right) {
                    Ok(self.builder.build_int_compare(inkwell::IntPredicate::SLT, l, r, "lt").into())
                } else {
                    Err(JitError::CompilationFailed("Type mismatch in comparison".to_string()))
                }
            }
            _ => Err(JitError::CompilationFailed(format!("Unsupported binary operator: {}", op)))
        }
    }
    
    /// Compile a function call
    fn compile_function_call(&mut self, function_name: &str, args: &[Expr], local_symbols: &HashMap<String, BasicValueEnum<'ctx>>) -> JitResult<BasicValueEnum<'ctx>> {
        let function = self.functions.get(function_name)
            .ok_or_else(|| JitError::CompilationFailed(format!("Function {} not found", function_name)))?
            .clone();
        
        // Compile arguments
        let arg_values: Result<Vec<_>, _> = args.iter()
            .map(|arg| self.compile_expression(arg, local_symbols))
            .collect();
        let arg_values = arg_values?;
        
        // Convert to BasicMetadataValueEnum for call
        let call_args: Vec<_> = arg_values.iter()
            .map(|val| (*val).into())
            .collect();
        
        // Build function call
        let call_result = self.builder.build_call(function, &call_args, "call");
        
        if let Some(return_value) = call_result.try_as_basic_value().left() {
            Ok(return_value)
        } else {
            // Void function
            Ok(self.context.i32_type().const_int(0, false).into())
        }
    }
    
    /// Convert Vāktra type to LLVM type
    fn convert_type(&self, vaaktra_type: &Type) -> JitResult<BasicTypeEnum<'ctx>> {
        match vaaktra_type {
            Type::Named(path, _span) => {
                let name = path.segments.last().unwrap().as_str();
                match name {
                    "सङ्ख्या" => Ok(self.context.i64_type().into()),
                    "सत्यासत्य" => Ok(self.context.bool_type().into()),
                    "शब्द" => Ok(self.context.i8_type().ptr_type(AddressSpace::Generic).into()),
                    "शून्य" => Ok(self.context.i32_type().into()), // Use i32 for void
                    _ => Err(JitError::CompilationFailed(format!("Unknown type: {}", name)))
                }
            }
            _ => Err(JitError::CompilationFailed("Unsupported type".to_string()))
        }
    }
    
    /// Get default value for a type
    fn get_default_value(&self, llvm_type: BasicTypeEnum<'ctx>) -> BasicValueEnum<'ctx> {
        match llvm_type {
            BasicTypeEnum::IntType(int_type) => int_type.const_int(0, false).into(),
            BasicTypeEnum::FloatType(float_type) => float_type.const_float(0.0).into(),
            BasicTypeEnum::PointerType(ptr_type) => ptr_type.const_null().into(),
            BasicTypeEnum::ArrayType(array_type) => array_type.const_zero().into(),
            BasicTypeEnum::StructType(struct_type) => struct_type.const_zero().into(),
            BasicTypeEnum::VectorType(vector_type) => vector_type.const_zero().into(),
        }
    }
    
    /// Get a JIT-compiled function
    pub fn get_jit_function<F>(&self, name: &str) -> JitResult<JitFunction<F>>
    where
        F: UnsafeFunctionPointer,
    {
        unsafe {
            self.execution_engine
                .get_function(name)
                .map_err(|e| JitError::CompilationFailed(format!("Failed to get JIT function: {}", e)))
        }
    }
    
    /// Set optimization level
    pub fn set_optimization_level(&mut self, level: OptimizationLevel) {
        self.optimization_level = level;
    }
    
    /// Dump LLVM IR for debugging
    pub fn dump_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }
}

/// Trait for unsafe function pointers (required by inkwell)
pub unsafe trait UnsafeFunctionPointer {}

unsafe impl UnsafeFunctionPointer for unsafe extern "C" fn() -> i64 {}
unsafe impl UnsafeFunctionPointer for unsafe extern "C" fn(i64) -> i64 {}
unsafe impl UnsafeFunctionPointer for unsafe extern "C" fn(i64, i64) -> i64 {}
