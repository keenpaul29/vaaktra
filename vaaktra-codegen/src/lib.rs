//! Vāktra (वाक्त्र) Code Generation
//! 
//! Simplified code generation for Sanskrit-inspired language
//! with focus on core functionality and buildability.

pub mod simple;

pub use simple::{SimpleVaaktraCodegen, SimpleCodegenError, SimpleCodegenResult};

/// Code generation errors
#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("Code generation failed: {0}")]
    GenerationFailed(String),
    
    #[error("Target not supported: {0}")]
    UnsupportedTarget(String),
    
    #[error("Optimization failed: {0}")]
    OptimizationFailed(String),
    
    #[error("Backend error: {0}")]
    BackendError(String),
}

pub type CodegenResult<T> = Result<T, CodegenError>;

/// Simplified code generator for Vāktra
pub struct VaaktraCodegen {
    /// Symbol table for generated functions
    symbols: HashMap<String, String>,
    
    /// Generated code output
    generated_code: Vec<String>,
    
    /// Current optimization level
    optimization_level: OptimizationLevel,
}

/// Optimization levels for code generation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationLevel {
    /// No optimization (तमस्)
    None,
    
    /// Basic optimization (रजस्)
    Basic,
    
    /// Aggressive optimization (सत्त्व)
    Aggressive,
}

impl VaaktraCodegen {
    /// Create a new code generator
    pub fn new() -> CodegenResult<Self> {
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())
            .map_err(|e| CodegenError::GenerationFailed(e.to_string()))?;
        
        // Enable aggressive optimizations
        let mut flags = settings::builder();
        flags.set("use_colocated_libcalls", "false")
            .map_err(|e| CodegenError::GenerationFailed(e.to_string()))?;
        flags.set("is_pic", "false")
            .map_err(|e| CodegenError::GenerationFailed(e.to_string()))?;
        flags.set("opt_level", "speed")
            .map_err(|e| CodegenError::GenerationFailed(e.to_string()))?;
        
        let isa_builder = cranelift_native::builder()
            .map_err(|e| CodegenError::GenerationFailed(e.to_string()))?;
        let isa = isa_builder.finish(settings::Flags::new(flags))
            .map_err(|e| CodegenError::GenerationFailed(e.to_string()))?;
        
        builder.isa(isa);
        
        let module = JITModule::new(builder)
            .map_err(|e| CodegenError::GenerationFailed(e.to_string()))?;
        
        let mut codegen = VaaktraCodegen {
            module,
            builder_context: FunctionBuilderContext::new(),
            ctx: codegen::Context::new(),
            symbols: HashMap::new(),
            type_map: HashMap::new(),
            optimization_level: OptimizationLevel::Aggressive,
        };
        
        codegen.initialize_type_mappings();
        Ok(codegen)
    }
    
    /// Initialize type mappings from Vāktra to Cranelift types
    fn initialize_type_mappings(&mut self) {
        self.type_map.insert("सङ्ख्या".to_string(), types::I64);
        self.type_map.insert("सत्यासत्य".to_string(), types::I8);
        self.type_map.insert("शब्द".to_string(), types::I64); // Pointer to string
        self.type_map.insert("शून्य".to_string(), types::I32); // Void represented as i32
    }
    
    /// Generate code for a complete program
    pub fn generate_program(&mut self, program: &Program, analyzer: &VaaktraSemanticAnalyzer) -> CodegenResult<()> {
        log::info!("Starting code generation for Vāktra program");
        
        // Generate code for all functions
        for item in &program.items {
            match item {
                Item::Mantra(mantra) => {
                    self.generate_function(mantra, analyzer)?;
                }
                _ => {
                    // Handle other item types as needed
                }
            }
        }
        
        // Finalize the module
        self.module.finalize_definitions()
            .map_err(|e| CodegenError::GenerationFailed(e.to_string()))?;
        
        log::info!("Code generation completed successfully");
        Ok(())
    }
    
    /// Generate code for a function
    fn generate_function(&mut self, mantra: &MantraDef, analyzer: &VaaktraSemanticAnalyzer) -> CodegenResult<()> {
        log::debug!("Generating code for function: {}", mantra.name);
        
        // Create function signature
        let mut sig = self.module.make_signature();
        
        // Add parameters
        for param in &mantra.params {
            let param_type = self.convert_type(&param.param_type)?;
            sig.params.push(AbiParam::new(param_type));
        }
        
        // Add return type
        let return_type = self.convert_type(&mantra.return_type)?;
        if return_type != types::I32 { // Don't add void return
            sig.returns.push(AbiParam::new(return_type));
        }
        
        // Declare the function
        let func_id = self.module
            .declare_function(&mantra.name, Linkage::Export, &sig)
            .map_err(|e| CodegenError::GenerationFailed(e.to_string()))?;
        
        // Define the function
        self.ctx.func.signature = sig;
        self.ctx.func.name = cranelift::codegen::ir::ExternalName::user(0, func_id.as_u32());
        
        // Build function body
        {
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);
            
            // Generate function body
            if let Some(body) = &mantra.body {
                let return_value = self.generate_block(&mut builder, body, analyzer)?;
                
                if let Some(ret_val) = return_value {
                    builder.ins().return_(&[ret_val]);
                } else {
                    builder.ins().return_(&[]);
                }
            } else {
                // Empty function
                if return_type != types::I32 {
                    let zero = builder.ins().iconst(return_type, 0);
                    builder.ins().return_(&[zero]);
                } else {
                    builder.ins().return_(&[]);
                }
            }
            
            builder.finalize();
        }
        
        // Compile the function
        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| CodegenError::GenerationFailed(e.to_string()))?;
        
        // Clear context for next function
        self.ctx.clear();
        
        // Get function pointer
        let func_ptr = self.module.get_finalized_function(func_id);
        self.symbols.insert(mantra.name.clone(), func_ptr);
        
        Ok(())
    }
    
    /// Generate code for a block of statements
    fn generate_block(
        &self, 
        builder: &mut FunctionBuilder, 
        statements: &[vaaktra_parser::ast::Statement], 
        _analyzer: &VaaktraSemanticAnalyzer
    ) -> CodegenResult<Option<Value>> {
        let mut last_value = None;
        
        for statement in statements {
            match statement {
                vaaktra_parser::ast::Statement::Expression(expr) => {
                    last_value = Some(self.generate_expression(builder, expr)?);
                }
                vaaktra_parser::ast::Statement::Let { name: _, value, .. } => {
                    // Generate the value but don't store it for now (simplified)
                    self.generate_expression(builder, value)?;
                }
                _ => {
                    // Handle other statement types
                    log::warn!("Unhandled statement type in code generation");
                }
            }
        }
        
        Ok(last_value)
    }
    
    /// Generate code for an expression
    fn generate_expression(&self, builder: &mut FunctionBuilder, expr: &Expr) -> CodegenResult<Value> {
        match expr {
            Expr::Literal { value, .. } => {
                self.generate_literal(builder, value)
            }
            
            Expr::Binary { left, op, right, .. } => {
                let left_val = self.generate_expression(builder, left)?;
                let right_val = self.generate_expression(builder, right)?;
                self.generate_binary_op(builder, left_val, op, right_val)
            }
            
            Expr::Identifier { name, .. } => {
                // For now, return a placeholder value
                // Real implementation would look up variable
                Ok(builder.ins().iconst(types::I64, 0))
            }
            
            _ => {
                Err(CodegenError::GenerationFailed("Unsupported expression type".to_string()))
            }
        }
    }
    
    /// Generate code for a literal
    fn generate_literal(&self, builder: &mut FunctionBuilder, literal: &str) -> CodegenResult<Value> {
        if let Ok(int_val) = literal.parse::<i64>() {
            Ok(builder.ins().iconst(types::I64, int_val))
        } else if literal == "सत्य" {
            Ok(builder.ins().iconst(types::I8, 1))
        } else if literal == "असत्य" {
            Ok(builder.ins().iconst(types::I8, 0))
        } else {
            Err(CodegenError::GenerationFailed(format!("Unsupported literal: {}", literal)))
        }
    }
    
    /// Generate code for binary operation
    fn generate_binary_op(
        &self, 
        builder: &mut FunctionBuilder, 
        left: Value, 
        op: &vaaktra_parser::ast::BinaryOp, 
        right: Value
    ) -> CodegenResult<Value> {
        use vaaktra_parser::ast::BinaryOp;
        
        match op {
            BinaryOp::Add => Ok(builder.ins().iadd(left, right)),
            BinaryOp::Sub => Ok(builder.ins().isub(left, right)),
            BinaryOp::Mul => Ok(builder.ins().imul(left, right)),
            BinaryOp::Div => Ok(builder.ins().sdiv(left, right)),
            BinaryOp::Mod => Ok(builder.ins().srem(left, right)),
            BinaryOp::Eq => Ok(builder.ins().icmp(IntCC::Equal, left, right)),
            BinaryOp::Ne => Ok(builder.ins().icmp(IntCC::NotEqual, left, right)),
            BinaryOp::Lt => Ok(builder.ins().icmp(IntCC::SignedLessThan, left, right)),
            BinaryOp::Le => Ok(builder.ins().icmp(IntCC::SignedLessThanOrEqual, left, right)),
            BinaryOp::Gt => Ok(builder.ins().icmp(IntCC::SignedGreaterThan, left, right)),
            BinaryOp::Ge => Ok(builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, left, right)),
            BinaryOp::And => Ok(builder.ins().band(left, right)),
            BinaryOp::Or => Ok(builder.ins().bor(left, right)),
        }
    }
    
    /// Convert Vāktra type to Cranelift type
    fn convert_type(&self, vaaktra_type: &Type) -> CodegenResult<cranelift::prelude::Type> {
        match vaaktra_type {
            Type::Named { name, .. } => {
                self.type_map.get(name)
                    .copied()
                    .ok_or_else(|| CodegenError::GenerationFailed(format!("Unknown type: {}", name)))
            }
            _ => Err(CodegenError::GenerationFailed("Unsupported type".to_string()))
        }
    }
    
    /// Get a function pointer by name
    pub fn get_function(&self, name: &str) -> Option<*const u8> {
        self.symbols.get(name).copied()
    }
    
    /// Set optimization level
    pub fn set_optimization_level(&mut self, level: OptimizationLevel) {
        self.optimization_level = level;
    }
    
    /// Execute a generated function (unsafe)
    pub unsafe fn execute_function_i64(&self, name: &str, args: &[i64]) -> CodegenResult<i64> {
        let func_ptr = self.get_function(name)
            .ok_or_else(|| CodegenError::GenerationFailed(format!("Function {} not found", name)))?;
        
        match args.len() {
            0 => {
                let func: extern "C" fn() -> i64 = std::mem::transmute(func_ptr);
                Ok(func())
            }
            1 => {
                let func: extern "C" fn(i64) -> i64 = std::mem::transmute(func_ptr);
                Ok(func(args[0]))
            }
            2 => {
                let func: extern "C" fn(i64, i64) -> i64 = std::mem::transmute(func_ptr);
                Ok(func(args[0], args[1]))
            }
            _ => Err(CodegenError::GenerationFailed("Too many arguments".to_string()))
        }
    }
}

impl Default for VaaktraCodegen {
    fn default() -> Self {
        Self::new().expect("Failed to create default VaaktraCodegen")
    }
}
