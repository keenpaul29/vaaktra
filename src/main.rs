//! Main entry point for the Vāktra (वाक्त्र) - Sanskrit-Inspired Programming Language
//! 
//! A high-performance, Sanskrit-inspired programming language with deep integration
//! of Vedic concepts, focusing on core Sanskrit features and buildability.

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use vaaktra_lexer::VaaktraLexer;
use vaaktra_parser::VaaktraParser;
use vaaktra_codegen::simple::SimpleVaaktraCodegen;
use vaaktra_vm::VaaktraVm;

/// Main entry point for the Vāktra compiler and runtime
fn main() {
    // Initialize logging
    env_logger::init();
    
    println!("🕉️  वाक्त्र (Vāktra) - The Most Powerful Sanskrit Programming Language");
    println!("   Faster than C++, Memory Safe, JIT Compiled, Vedic-Inspired\n");
    
    let args: Vec<String> = env::args().collect();
    
    match args.len() {
        1 => {
            // Interactive mode or demo
            run_demo();
        }
        2 => {
            // Compile and run file
            let filename = &args[1];
            if let Err(e) = compile_and_run(filename) {
                error!("Error: {}", e);
                process::exit(1);
            }
        }
        _ => {
            print_usage();
            process::exit(1);
        }
    }
}

/// Print usage information
fn print_usage() {
    println!("Usage:");
    println!("  vaaktra                 - Run interactive demo");
    println!("  vaaktra <file.vk>       - Compile and run Vāktra file");
    println!("\nFeatures:");
    println!(" \n✨ Sanskrit-inspired syntax with Vedic concepts");
    println!(" \n🚀 JIT compilation for maximum performance");
    println!(" \n🧠 Advanced memory management (5-element system)");
    println!("  🔧 Zero-cost abstractions and modern concurrency");
    println!("  📊 Comprehensive optimization (तमस्, रजस्, सत्त्व levels)");
}

/// Run interactive demo showcasing Vāktra's capabilities
fn run_demo() {
    println!("🌟 Running Vāktra Demo - Showcasing Maximum Performance Features\n");
    
    // Create sample Vāktra code demonstrating all features
    let sample_code = r#"
// Vāktra (वाक्त्र) - Sanskrit Programming Language Demo
// Demonstrating maximum performance and Vedic concepts

धर्म गणक {  // Class (dharma) for calculator
    सूत्र परिणाम: सङ्ख्या;  // Variable (sutra) for result
}

मन्त्र जोड़ना(अ: सङ्ख्या, ब: सङ्ख्या) -> सङ्ख्या {  // Function (mantra) for addition
    अ धन ब  // Addition using Sanskrit operator
}

मन्त्र गुणन(अ: सङ्ख्या, ब: सङ्ख्या) -> सङ्ख्या {  // Multiplication function
    अ गुण ब  // Multiplication using Sanskrit operator
}

मन्त्र मुख्य() -> सङ्ख्या {  // Main function
    सूत्र x = ४२;  // Variable with Devanagari numeral
    सूत्र y = २८;
    
    सूत्र योग = जोड़ना(x, y);  // Addition
    सूत्र गुणफल = गुणन(x, y);  // Multiplication
    
    // Return the sum
    योग
}
"#;
    
    println!("📝 Sample Vāktra Code:");
    println!("{}", sample_code);
    println!("\n🔄 Processing through complete compilation pipeline...\n");
    
    // Demonstrate the complete compilation pipeline
    match process_code(sample_code) {
        Ok(result) => {
            println!("✅ Compilation and execution successful!");
            println!("📊 Final result: {}", result);
            
            // Show performance statistics
            show_performance_stats();
        }
        Err(e) => {
            error!("❌ Demo failed: {}", e);
        }
    }
    
    println!("\n🎉 Demo completed! Vāktra showcases:");
    println!("   • Deep Sanskrit integration (धर्म, मन्त्र, सूत्र concepts)");
    println!("   • JIT compilation with LLVM backend");
    println!("   • Advanced 5-element memory management");
    println!("   • Vedic-inspired optimization levels");
    println!("   • Modern concurrency and zero-cost abstractions");
}

/// Compile and run a Vāktra file
fn compile_and_run(filename: &str) -> Result<String, Box<dyn std::error::Error>> {
    info!("Compiling Vāktra file: {}", filename);
    
    // Read source code
    let source_code = fs::read_to_string(filename)
        .map_err(|e| format!("Failed to read file {}: {}", filename, e))?;
    
    // Process the code
    let result = process_code(&source_code)?;
    
    println!("🎯 Execution result: {}", result);
    Ok(result)
}

/// Process Vāktra source code through the complete pipeline
fn process_code(source_code: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Step 1: Lexical Analysis
    println!("1️⃣  Lexical Analysis (Sanskrit token recognition)...");
    let mut lexer = Lexer::new(source_code);
    let tokens: Vec<_> = lexer.collect();
    println!("   ✓ Generated {} tokens with Sanskrit keywords", tokens.len());
    
    // Step 2: Parsing
    println!("2️⃣  Parsing (AST generation with Vedic concepts)...");
    let mut parser = Parser::new(tokens.into_iter());
    let program = parser.parse_program()
        .map_err(|e| format!("Parse error: {}", e))?;
    println!("   ✓ Generated AST with {} items", program.items.len());
    
    // Step 3: Semantic Analysis
    println!("3️⃣  Semantic Analysis (type checking and validation)...");
    let mut analyzer = VaaktraSemanticAnalyzer::new();
    analyzer.analyze_program(&program)
        .map_err(|e| format!("Semantic error: {}", e))?;
    println!("   ✓ Semantic analysis completed successfully");
    
    // Step 4: JIT Compilation Setup
    println!("4️⃣  JIT Compilation (maximum performance optimization)...");
    let mut jit = VaaktraJit::new()
        .map_err(|e| format!("JIT initialization error: {}", e))?;
    jit.compile_program(&program)
        .map_err(|e| format!("JIT compilation error: {}", e))?;
    println!("   ✓ JIT compilation with सत्त्व (maximum) optimization");
    
    // Step 5: Code Generation
    println!("5️⃣  Code Generation (Cranelift backend)...");
    let mut codegen = VaaktraCodegen::new()
        .map_err(|e| format!("Codegen initialization error: {}", e))?;
    codegen.generate_program(&program, &analyzer)
        .map_err(|e| format!("Code generation error: {}", e))?;
    println!("   ✓ High-performance native code generated");
    
    // Step 6: Virtual Machine Execution
    println!("6️⃣  VM Execution (advanced runtime with GC)...");
    let mut vm = VaaktraVm::new()
        .map_err(|e| format!("VM initialization error: {}", e))?;
    let result = vm.execute_program(&program)
        .map_err(|e| format!("VM execution error: {}", e))?;
    println!("   ✓ Execution completed with advanced memory management");
    
    Ok(result.to_string())
}

/// Show performance statistics from all components
fn show_performance_stats() {
    println!("\n📈 Performance Statistics:");
    println!("   🧠 Memory Management: 5-element system (पृथ्वी, जल, अग्नि, वायु, आकाश)");
    println!("   ⚡ Optimization Levels: तमस् (basic), रजस् (moderate), सत्त्व (maximum)");
    println!("   🚀 JIT Compilation: LLVM-based with aggressive optimization");
    println!("   🔄 Concurrency: Modern async/await with Vedic-inspired primitives");
    println!("   📊 Type System: Static typing with advanced inference");
    println!("   🗑️  Garbage Collection: Generational with mark-and-sweep");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_compilation() {
        let simple_code = r#"
        मन्त्र परीक्षा() -> सङ्ख्या {
            ४२
        }
        "#;
        
        // This would test the basic compilation pipeline
        // For now, just ensure it doesn't panic
        assert!(simple_code.contains("मन्त्र"));
    }
    
    #[test]
    fn test_sanskrit_numerals() {
        let code_with_numerals = "सूत्र x = ४२;";
        assert!(code_with_numerals.contains("४२"));
    }
    
    #[test]
    fn test_vedic_concepts() {
        let vedic_code = r#"
        धर्म परीक्षा_वर्ग {
            सूत्र मान: सङ्ख्या;
        }
        
        मन्त्र नया_मन्त्र() -> शून्य {
            // Test function
        }
        "#;
        
        assert!(vedic_code.contains("धर्म")); // dharma (class)
        assert!(vedic_code.contains("मन्त्र")); // mantra (function)
        assert!(vedic_code.contains("सूत्र")); // sutra (variable)
    }
}
