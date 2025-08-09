//! Bytecode Interpreter for Vāktra VM
//! 
//! High-performance interpreter with advanced execution strategies
//! inspired by Vedic concepts of understanding (बोध) and realization (साक्षात्कार).

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::{VmError, VmResult};
use crate::bytecode::{BytecodeProgram, Instruction, Constant, FunctionInfo};
use crate::stack::{VmStack, CallFrame};
use vaaktra_jit::runtime::RuntimeValue;
use vaaktra_parser::ast::{Program, Item, MantraDef, Expr, Statement, BinaryOp};

/// Bytecode interpreter
pub struct BytecodeInterpreter {
    /// Instruction pointer
    ip: u32,
    
    /// Current bytecode program
    program: Option<BytecodeProgram>,
    
    /// Execution statistics
    stats: InterpreterStats,
}

/// Interpreter execution statistics
#[derive(Debug, Default)]
pub struct InterpreterStats {
    pub instructions_executed: u64,
    pub function_calls: u64,
    pub jumps_taken: u64,
    pub memory_allocations: u64,
}

impl BytecodeInterpreter {
    /// Create a new bytecode interpreter
    pub fn new() -> VmResult<Self> {
        Ok(BytecodeInterpreter {
            ip: 0,
            program: None,
            stats: InterpreterStats::default(),
        })
    }
    
    /// Compile a Vāktra program to bytecode
    pub fn compile_program(&mut self, program: &Program) -> VmResult<BytecodeProgram> {
        log::debug!("Compiling Vāktra program to bytecode");
        
        let mut bytecode = BytecodeProgram::new();
        let mut compiler = BytecodeCompiler::new();
        
        // Compile all items
        for item in &program.items {
            match item {
                Item::Mantra(mantra) => {
                    compiler.compile_function(mantra, &mut bytecode)?;
                }
                _ => {
                    // Handle other item types
                    log::warn!("Unhandled item type in bytecode compilation");
                }
            }
        }
        
        // Set entry point (main function if exists)
        if let Some(main_func) = bytecode.get_function("main") {
            bytecode.entry_point = main_func.start_address;
        }
        
        self.program = Some(bytecode.clone());
        Ok(bytecode)
    }
    
    /// Execute bytecode program
    pub fn execute(
        &mut self, 
        program: &BytecodeProgram, 
        stack: &mut VmStack, 
        globals: &Arc<RwLock<HashMap<String, RuntimeValue>>>
    ) -> VmResult<RuntimeValue> {
        log::debug!("Starting bytecode execution");
        
        self.program = Some(program.clone());
        self.ip = program.entry_point;
        
        loop {
            let instruction = program.get_instruction(self.ip)
                .ok_or_else(|| VmError::InvalidBytecode(format!("Invalid instruction pointer: {}", self.ip)))?;
            
            self.stats.instructions_executed += 1;
            
            match self.execute_instruction(instruction, stack, globals)? {
                ExecutionResult::Continue => {
                    self.ip += 1;
                }
                ExecutionResult::Jump(addr) => {
                    self.ip = addr;
                    self.stats.jumps_taken += 1;
                }
                ExecutionResult::Return(value) => {
                    if stack.call_depth() == 0 {
                        // Program finished
                        return Ok(value);
                    } else {
                        // Return from function call
                        let frame = stack.pop_frame()?;
                        self.ip = frame.return_address;
                        stack.push(value)?;
                    }
                }
                ExecutionResult::Halt(value) => {
                    return Ok(value);
                }
            }
        }
    }
    
    /// Execute a single instruction
    fn execute_instruction(
        &mut self,
        instruction: &Instruction,
        stack: &mut VmStack,
        globals: &Arc<RwLock<HashMap<String, RuntimeValue>>>
    ) -> VmResult<ExecutionResult> {
        match instruction {
            // Stack operations
            Instruction::PushConst(constant) => {
                let value = self.constant_to_runtime_value(constant);
                stack.push(value)?;
                Ok(ExecutionResult::Continue)
            }
            
            Instruction::Pop => {
                stack.pop()?;
                Ok(ExecutionResult::Continue)
            }
            
            Instruction::Dup => {
                stack.dup()?;
                Ok(ExecutionResult::Continue)
            }
            
            Instruction::Swap => {
                stack.swap()?;
                Ok(ExecutionResult::Continue)
            }
            
            // Arithmetic operations
            Instruction::Add => {
                let b = stack.pop()?;
                let a = stack.pop()?;
                let result = self.add_values(a, b)?;
                stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }
            
            Instruction::Sub => {
                let b = stack.pop()?;
                let a = stack.pop()?;
                let result = self.sub_values(a, b)?;
                stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }
            
            Instruction::Mul => {
                let b = stack.pop()?;
                let a = stack.pop()?;
                let result = self.mul_values(a, b)?;
                stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }
            
            Instruction::Div => {
                let b = stack.pop()?;
                let a = stack.pop()?;
                let result = self.div_values(a, b)?;
                stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }
            
            // Comparison operations
            Instruction::Eq => {
                let b = stack.pop()?;
                let a = stack.pop()?;
                let result = RuntimeValue::Satyasatya(self.values_equal(&a, &b));
                stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }
            
            Instruction::Lt => {
                let b = stack.pop()?;
                let a = stack.pop()?;
                let result = RuntimeValue::Satyasatya(self.less_than(&a, &b)?);
                stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }
            
            // Control flow
            Instruction::Jump(addr) => {
                Ok(ExecutionResult::Jump(*addr))
            }
            
            Instruction::JumpIf(addr) => {
                let condition = stack.pop()?;
                if condition.is_truthy() {
                    Ok(ExecutionResult::Jump(*addr))
                } else {
                    Ok(ExecutionResult::Continue)
                }
            }
            
            Instruction::JumpIfNot(addr) => {
                let condition = stack.pop()?;
                if !condition.is_truthy() {
                    Ok(ExecutionResult::Jump(*addr))
                } else {
                    Ok(ExecutionResult::Continue)
                }
            }
            
            Instruction::Call(func_name, arg_count) => {
                self.stats.function_calls += 1;
                
                // Get function info
                let program = self.program.as_ref().unwrap();
                let func_info = program.get_function(func_name)
                    .ok_or_else(|| VmError::ExecutionError(format!("Function {} not found", func_name)))?;
                
                // Create call frame
                let frame = CallFrame {
                    return_address: self.ip + 1,
                    base_pointer: stack.size() - *arg_count as usize,
                    function_name: func_name.clone(),
                    local_count: func_info.local_count,
                };
                
                stack.push_frame(frame)?;
                Ok(ExecutionResult::Jump(func_info.start_address))
            }
            
            Instruction::Return => {
                let return_value = if stack.is_empty() {
                    RuntimeValue::Shunya
                } else {
                    stack.pop()?
                };
                Ok(ExecutionResult::Return(return_value))
            }
            
            // Variable operations
            Instruction::LoadLocal(index) => {
                let value = stack.get_local(*index)?.clone();
                stack.push(value)?;
                Ok(ExecutionResult::Continue)
            }
            
            Instruction::StoreLocal(index) => {
                let value = stack.pop()?;
                stack.set_local(*index, value)?;
                Ok(ExecutionResult::Continue)
            }
            
            Instruction::LoadGlobal(name) => {
                let globals_read = globals.read();
                let value = globals_read.get(name)
                    .cloned()
                    .unwrap_or(RuntimeValue::Shunya);
                drop(globals_read);
                stack.push(value)?;
                Ok(ExecutionResult::Continue)
            }
            
            Instruction::StoreGlobal(name) => {
                let value = stack.pop()?;
                let mut globals_write = globals.write();
                globals_write.insert(name.clone(), value);
                Ok(ExecutionResult::Continue)
            }
            
            // Special operations
            Instruction::Print => {
                let value = stack.pop()?;
                println!("{}", value.to_string());
                Ok(ExecutionResult::Continue)
            }
            
            Instruction::Halt => {
                let value = if stack.is_empty() {
                    RuntimeValue::Shunya
                } else {
                    stack.pop()?
                };
                Ok(ExecutionResult::Halt(value))
            }
            
            Instruction::Nop => {
                Ok(ExecutionResult::Continue)
            }
            
            _ => {
                Err(VmError::InvalidBytecode(format!("Unimplemented instruction: {:?}", instruction)))
            }
        }
    }
    
    /// Convert constant to runtime value
    fn constant_to_runtime_value(&self, constant: &Constant) -> RuntimeValue {
        match constant {
            Constant::Integer(i) => RuntimeValue::Sankhya(*i),
            Constant::Boolean(b) => RuntimeValue::Satyasatya(*b),
            Constant::String(s) => RuntimeValue::Shabda(s.clone()),
            Constant::Null => RuntimeValue::Shunya,
        }
    }
    
    /// Add two runtime values
    fn add_values(&self, a: RuntimeValue, b: RuntimeValue) -> VmResult<RuntimeValue> {
        match (a, b) {
            (RuntimeValue::Sankhya(x), RuntimeValue::Sankhya(y)) => {
                Ok(RuntimeValue::Sankhya(x + y))
            }
            (RuntimeValue::Shabda(x), RuntimeValue::Shabda(y)) => {
                Ok(RuntimeValue::Shabda(format!("{}{}", x, y)))
            }
            _ => Err(VmError::ExecutionError("Invalid operands for addition".to_string()))
        }
    }
    
    /// Subtract two runtime values
    fn sub_values(&self, a: RuntimeValue, b: RuntimeValue) -> VmResult<RuntimeValue> {
        match (a, b) {
            (RuntimeValue::Sankhya(x), RuntimeValue::Sankhya(y)) => {
                Ok(RuntimeValue::Sankhya(x - y))
            }
            _ => Err(VmError::ExecutionError("Invalid operands for subtraction".to_string()))
        }
    }
    
    /// Multiply two runtime values
    fn mul_values(&self, a: RuntimeValue, b: RuntimeValue) -> VmResult<RuntimeValue> {
        match (a, b) {
            (RuntimeValue::Sankhya(x), RuntimeValue::Sankhya(y)) => {
                Ok(RuntimeValue::Sankhya(x * y))
            }
            _ => Err(VmError::ExecutionError("Invalid operands for multiplication".to_string()))
        }
    }
    
    /// Divide two runtime values
    fn div_values(&self, a: RuntimeValue, b: RuntimeValue) -> VmResult<RuntimeValue> {
        match (a, b) {
            (RuntimeValue::Sankhya(x), RuntimeValue::Sankhya(y)) => {
                if y == 0 {
                    Err(VmError::ExecutionError("Division by zero".to_string()))
                } else {
                    Ok(RuntimeValue::Sankhya(x / y))
                }
            }
            _ => Err(VmError::ExecutionError("Invalid operands for division".to_string()))
        }
    }
    
    /// Check if two values are equal
    fn values_equal(&self, a: &RuntimeValue, b: &RuntimeValue) -> bool {
        match (a, b) {
            (RuntimeValue::Sankhya(x), RuntimeValue::Sankhya(y)) => x == y,
            (RuntimeValue::Satyasatya(x), RuntimeValue::Satyasatya(y)) => x == y,
            (RuntimeValue::Shabda(x), RuntimeValue::Shabda(y)) => x == y,
            (RuntimeValue::Shunya, RuntimeValue::Shunya) => true,
            _ => false,
        }
    }
    
    /// Check if first value is less than second
    fn less_than(&self, a: &RuntimeValue, b: &RuntimeValue) -> VmResult<bool> {
        match (a, b) {
            (RuntimeValue::Sankhya(x), RuntimeValue::Sankhya(y)) => Ok(x < y),
            (RuntimeValue::Shabda(x), RuntimeValue::Shabda(y)) => Ok(x < y),
            _ => Err(VmError::ExecutionError("Invalid operands for comparison".to_string()))
        }
    }
    
    /// Get interpreter statistics
    pub fn get_stats(&self) -> &InterpreterStats {
        &self.stats
    }
}

/// Result of instruction execution
#[derive(Debug)]
enum ExecutionResult {
    Continue,
    Jump(u32),
    Return(RuntimeValue),
    Halt(RuntimeValue),
}

/// Bytecode compiler
struct BytecodeCompiler {
    // Compiler state would go here
}

impl BytecodeCompiler {
    fn new() -> Self {
        BytecodeCompiler {}
    }
    
    fn compile_function(&mut self, mantra: &MantraDef, bytecode: &mut BytecodeProgram) -> VmResult<()> {
        let start_addr = bytecode.instructions.len() as u32;
        
        // Create function info
        let func_info = FunctionInfo {
            name: mantra.name.clone(),
            start_address: start_addr,
            param_count: mantra.params.len() as u8,
            local_count: 0, // Simplified - would need to analyze locals
            return_type: "unknown".to_string(), // Simplified
        };
        
        // Compile function body (simplified)
        if let Some(body) = &mantra.body {
            for statement in body {
                self.compile_statement(statement, bytecode)?;
            }
        }
        
        // Add return instruction
        bytecode.add_instruction(Instruction::Return);
        
        // Add function info
        bytecode.add_function(func_info);
        
        Ok(())
    }
    
    fn compile_statement(&mut self, statement: &Statement, bytecode: &mut BytecodeProgram) -> VmResult<()> {
        match statement {
            Statement::Expression(expr) => {
                self.compile_expression(expr, bytecode)?;
            }
            _ => {
                // Handle other statement types
                log::warn!("Unhandled statement type in bytecode compilation");
            }
        }
        Ok(())
    }
    
    fn compile_expression(&mut self, expr: &Expr, bytecode: &mut BytecodeProgram) -> VmResult<()> {
        match expr {
            Expr::Literal(literal, _span) => {
                match literal {
                    vaaktra_parser::ast::Literal::Int(value) => {
                        bytecode.add_instruction(Instruction::PushConst(Constant::Integer(*value)));
                    }
                    vaaktra_parser::ast::Literal::Bool(value) => {
                        bytecode.add_instruction(Instruction::PushConst(Constant::Boolean(*value)));
                    }
                    vaaktra_parser::ast::Literal::String(value) => {
                        bytecode.add_instruction(Instruction::PushConst(Constant::String(value.clone())));
                    }
                }
            }
            
            Expr::Binary(left, op, right, _span) => {
                self.compile_expression(left, bytecode)?;
                self.compile_expression(right, bytecode)?;
                
                match op {
                    BinaryOp::Add => bytecode.add_instruction(Instruction::Add),
                    BinaryOp::Sub => bytecode.add_instruction(Instruction::Sub),
                    BinaryOp::Mul => bytecode.add_instruction(Instruction::Mul),
                    BinaryOp::Div => bytecode.add_instruction(Instruction::Div),
                    BinaryOp::Eq => bytecode.add_instruction(Instruction::Eq),
                    BinaryOp::Lt => bytecode.add_instruction(Instruction::Lt),
                    _ => return Err(VmError::InvalidBytecode("Unsupported binary operator".to_string())),
                };
            }
            
            _ => {
                return Err(VmError::InvalidBytecode("Unsupported expression type".to_string()));
            }
        }
        
        Ok(())
    }
}
