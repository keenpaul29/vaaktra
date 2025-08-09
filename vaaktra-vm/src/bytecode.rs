//! Bytecode definitions for Vāktra VM
//! 
//! Defines the instruction set inspired by Vedic concepts
//! for maximum execution efficiency.

use serde::{Serialize, Deserialize};
use std::fmt;

/// Bytecode instruction set for Vāktra VM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    // === Stack Operations (स्तूप संचालन) ===
    /// Push constant value onto stack
    PushConst(Constant),
    
    /// Pop value from stack
    Pop,
    
    /// Duplicate top stack value
    Dup,
    
    /// Swap top two stack values
    Swap,
    
    // === Arithmetic Operations (गणित संचालन) ===
    /// Add two values (धन)
    Add,
    
    /// Subtract two values (ऋण)
    Sub,
    
    /// Multiply two values (गुण)
    Mul,
    
    /// Divide two values (भाग)
    Div,
    
    /// Modulo operation (शेष)
    Mod,
    
    /// Negate value
    Neg,
    
    // === Comparison Operations (तुलना संचालन) ===
    /// Equal comparison (समान)
    Eq,
    
    /// Not equal comparison (असमान)
    Ne,
    
    /// Less than (लघुत्तर)
    Lt,
    
    /// Less than or equal (समानता)
    Le,
    
    /// Greater than (महत्तर)
    Gt,
    
    /// Greater than or equal (महत्तर व समान)
    Ge,
    
    // === Logical Operations (तार्किक संचालन) ===
    /// Logical AND (च)
    And,
    
    /// Logical OR (वा)
    Or,
    
    /// Logical NOT (न)
    Not,
    
    // === Control Flow (नियंत्रण प्रवाह) ===
    /// Jump to address
    Jump(u32),
    
    /// Jump if true
    JumpIf(u32),
    
    /// Jump if false
    JumpIfNot(u32),
    
    /// Call function
    Call(String, u8), // function name, arg count
    
    /// Return from function
    Return,
    
    // === Variable Operations (चर संचालन) ===
    /// Load local variable
    LoadLocal(u16),
    
    /// Store local variable
    StoreLocal(u16),
    
    /// Load global variable
    LoadGlobal(String),
    
    /// Store global variable
    StoreGlobal(String),
    
    // === Memory Operations (स्मृति संचालन) ===
    /// Allocate memory
    Alloc(u32),
    
    /// Load from memory
    Load,
    
    /// Store to memory
    Store,
    
    // === Array Operations (सूची संचालन) ===
    /// Create array
    NewArray(u32),
    
    /// Get array element
    ArrayGet,
    
    /// Set array element
    ArraySet,
    
    /// Get array length
    ArrayLen,
    
    // === Object Operations (वस्तु संचालन) ===
    /// Create object
    NewObject(String), // class name
    
    /// Get field
    GetField(String),
    
    /// Set field
    SetField(String),
    
    // === Special Operations (विशेष संचालन) ===
    /// No operation
    Nop,
    
    /// Halt execution
    Halt,
    
    /// Print value (for debugging)
    Print,
}

/// Constant values in bytecode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constant {
    /// Integer constant (सङ्ख्या)
    Integer(i64),
    
    /// Boolean constant (सत्यासत्य)
    Boolean(bool),
    
    /// String constant (शब्द)
    String(String),
    
    /// Null/void constant (शून्य)
    Null,
}

/// Bytecode program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BytecodeProgram {
    /// Instructions
    pub instructions: Vec<Instruction>,
    
    /// Constant pool
    pub constants: Vec<Constant>,
    
    /// Function table
    pub functions: std::collections::HashMap<String, FunctionInfo>,
    
    /// Entry point
    pub entry_point: u32,
}

/// Function information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    /// Function name
    pub name: String,
    
    /// Start address in bytecode
    pub start_address: u32,
    
    /// Number of parameters
    pub param_count: u8,
    
    /// Number of local variables
    pub local_count: u16,
    
    /// Return type
    pub return_type: String,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::PushConst(c) => write!(f, "PUSH_CONST {:?}", c),
            Instruction::Pop => write!(f, "POP"),
            Instruction::Dup => write!(f, "DUP"),
            Instruction::Swap => write!(f, "SWAP"),
            Instruction::Add => write!(f, "ADD"),
            Instruction::Sub => write!(f, "SUB"),
            Instruction::Mul => write!(f, "MUL"),
            Instruction::Div => write!(f, "DIV"),
            Instruction::Mod => write!(f, "MOD"),
            Instruction::Neg => write!(f, "NEG"),
            Instruction::Eq => write!(f, "EQ"),
            Instruction::Ne => write!(f, "NE"),
            Instruction::Lt => write!(f, "LT"),
            Instruction::Le => write!(f, "LE"),
            Instruction::Gt => write!(f, "GT"),
            Instruction::Ge => write!(f, "GE"),
            Instruction::And => write!(f, "AND"),
            Instruction::Or => write!(f, "OR"),
            Instruction::Not => write!(f, "NOT"),
            Instruction::Jump(addr) => write!(f, "JUMP {}", addr),
            Instruction::JumpIf(addr) => write!(f, "JUMP_IF {}", addr),
            Instruction::JumpIfNot(addr) => write!(f, "JUMP_IF_NOT {}", addr),
            Instruction::Call(name, argc) => write!(f, "CALL {} {}", name, argc),
            Instruction::Return => write!(f, "RETURN"),
            Instruction::LoadLocal(idx) => write!(f, "LOAD_LOCAL {}", idx),
            Instruction::StoreLocal(idx) => write!(f, "STORE_LOCAL {}", idx),
            Instruction::LoadGlobal(name) => write!(f, "LOAD_GLOBAL {}", name),
            Instruction::StoreGlobal(name) => write!(f, "STORE_GLOBAL {}", name),
            Instruction::Alloc(size) => write!(f, "ALLOC {}", size),
            Instruction::Load => write!(f, "LOAD"),
            Instruction::Store => write!(f, "STORE"),
            Instruction::NewArray(size) => write!(f, "NEW_ARRAY {}", size),
            Instruction::ArrayGet => write!(f, "ARRAY_GET"),
            Instruction::ArraySet => write!(f, "ARRAY_SET"),
            Instruction::ArrayLen => write!(f, "ARRAY_LEN"),
            Instruction::NewObject(class) => write!(f, "NEW_OBJECT {}", class),
            Instruction::GetField(field) => write!(f, "GET_FIELD {}", field),
            Instruction::SetField(field) => write!(f, "SET_FIELD {}", field),
            Instruction::Nop => write!(f, "NOP"),
            Instruction::Halt => write!(f, "HALT"),
            Instruction::Print => write!(f, "PRINT"),
        }
    }
}

impl BytecodeProgram {
    /// Create a new empty bytecode program
    pub fn new() -> Self {
        BytecodeProgram {
            instructions: Vec::new(),
            constants: Vec::new(),
            functions: std::collections::HashMap::new(),
            entry_point: 0,
        }
    }
    
    /// Add an instruction
    pub fn add_instruction(&mut self, instruction: Instruction) -> u32 {
        let addr = self.instructions.len() as u32;
        self.instructions.push(instruction);
        addr
    }
    
    /// Add a constant
    pub fn add_constant(&mut self, constant: Constant) -> u32 {
        let idx = self.constants.len() as u32;
        self.constants.push(constant);
        idx
    }
    
    /// Add a function
    pub fn add_function(&mut self, info: FunctionInfo) {
        self.functions.insert(info.name.clone(), info);
    }
    
    /// Get instruction at address
    pub fn get_instruction(&self, addr: u32) -> Option<&Instruction> {
        self.instructions.get(addr as usize)
    }
    
    /// Get constant by index
    pub fn get_constant(&self, idx: u32) -> Option<&Constant> {
        self.constants.get(idx as usize)
    }
    
    /// Get function info
    pub fn get_function(&self, name: &str) -> Option<&FunctionInfo> {
        self.functions.get(name)
    }
    
    /// Disassemble the program for debugging
    pub fn disassemble(&self) -> String {
        let mut output = String::new();
        
        output.push_str("=== Vāktra Bytecode Disassembly ===\n\n");
        
        // Constants
        if !self.constants.is_empty() {
            output.push_str("Constants:\n");
            for (i, constant) in self.constants.iter().enumerate() {
                output.push_str(&format!("  {}: {:?}\n", i, constant));
            }
            output.push('\n');
        }
        
        // Functions
        if !self.functions.is_empty() {
            output.push_str("Functions:\n");
            for (name, info) in &self.functions {
                output.push_str(&format!("  {}: start={}, params={}, locals={}\n", 
                    name, info.start_address, info.param_count, info.local_count));
            }
            output.push('\n');
        }
        
        // Instructions
        output.push_str("Instructions:\n");
        for (i, instruction) in self.instructions.iter().enumerate() {
            output.push_str(&format!("  {:04}: {}\n", i, instruction));
        }
        
        output
    }
}

impl Default for BytecodeProgram {
    fn default() -> Self {
        Self::new()
    }
}
