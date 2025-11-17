use std::env;
use std::fs;
use std::process;

mod lexer;
mod parser;
mod semantic;
mod codegen;
mod error;

use lexer::Lexer;
use parser::Parser;
use semantic::SemanticAnalyzer;
use codegen::CodeGenerator;
use error::CompilerError;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: ssc <source_file> [-o <output_file>]");
        eprintln!("Options:");
        eprintln!("  -o <file>    Set output file name");
        eprintln!("  -O<level>    Set optimization level (0-3)");
        eprintln!("  --emit-ir    Emit intermediate representation");
        process::exit(1);
    }
    
    let source_file = &args[1];
    let mut output_file = "a.out";
    let mut opt_level = 0;
    let mut emit_ir = false;
    
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                if i + 1 < args.len() {
                    output_file = &args[i + 1];
                    i += 2;
                } else {
                    eprintln!("Error: -o requires an argument");
                    process::exit(1);
                }
            }
            arg if arg.starts_with("-O") => {
                if let Some(level) = arg.chars().nth(2) {
                    opt_level = level.to_digit(10).unwrap_or(0) as u8;
                }
                i += 1;
            }
            "--emit-ir" => {
                emit_ir = true;
                i += 1;
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                process::exit(1);
            }
        }
    }
    
    match compile(source_file, output_file, opt_level, emit_ir) {
        Ok(_) => {
            println!("Compilation successful: {}", output_file);
        }
        Err(e) => {
            eprintln!("Compilation failed: {}", e);
            process::exit(1);
        }
    }
}

fn compile(source_file: &str, output_file: &str, opt_level: u8, emit_ir: bool) -> Result<(), CompilerError> {
    println!("Compiling {}...", source_file);
    
    let source = fs::read_to_string(source_file)
        .map_err(|e| CompilerError::IoError(e.to_string()))?;
    
    println!("  [1/5] Lexical analysis...");
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize()?;
    
    println!("  [2/5] Parsing...");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    
    println!("  [3/5] Semantic analysis...");
    let mut semantic = SemanticAnalyzer::new();
    semantic.analyze(&ast)?;
    println!("       Semantic analysis completed successfully");
    
    println!("  [4/5] Code generation...");
    let mut codegen = CodeGenerator::new(opt_level);
    let ir = codegen.generate(&ast)?;
    println!("       Generated {} lines of IR", ir.lines().count());
    
    if emit_ir {
        let ir_file = format!("{}.ir", output_file);
        fs::write(&ir_file, &ir)
            .map_err(|e| CompilerError::IoError(e.to_string()))?;
        println!("       IR written to {}", ir_file);
    }
    
    println!("  [5/5] Assembling and linking...");
    let asm = codegen.to_assembly(&ast)?;
    println!("       Generated {} lines of assembly", asm.lines().count());
    
    let asm_file = format!("{}.asm", output_file);
    fs::write(&asm_file, &asm)
        .map_err(|e| CompilerError::IoError(e.to_string()))?;
    
    assemble_and_link(&asm_file, output_file)?;
    
    // Keep .asm file for debugging. Because we need it. :P
    // fs::remove_file(&asm_file).ok();
    
    Ok(())
}

fn assemble_and_link(asm_file: &str, output_file: &str) -> Result<(), CompilerError> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        let obj_file = format!("{}.obj", output_file);
        
        let nasm_output = Command::new("nasm")
            .args(&["-f", "win64", "-o", &obj_file, asm_file])
            .output();
        
        match nasm_output {
            Ok(output) => {
                if !output.status.success() {
                    return Err(CompilerError::AssemblyError(
                        String::from_utf8_lossy(&output.stderr).to_string()
                    ));
                }
            }
            Err(_) => {
                return Err(CompilerError::AssemblyError(
                    "NASM not found. Please install NASM assembler.".to_string()
                ));
            }
        }
        
        let link_output = Command::new("link")
            .args(&[
                "/SUBSYSTEM:CONSOLE",
                "/ENTRY:mainCRTStartup",
                &format!("/OUT:{}", output_file),
                &obj_file,
                "libcmt.lib",
                "libvcruntime.lib",
                "libucrt.lib",
                "kernel32.lib"
            ])
            .output();
        
        match link_output {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    return Err(CompilerError::LinkError(
                        format!("Linker failed:\nSTDOUT:\n{}\nSTDERR:\n{}", stdout, stderr)
                    ));
                }
            }
            Err(_) => {
                return Err(CompilerError::LinkError(
                    "Microsoft Linker not found. Please install Visual Studio.".to_string()
                ));
            }
        }
        
        fs::remove_file(&obj_file).ok();
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        return Err(CompilerError::LinkError(
            "Non-Windows platforms not yet supported".to_string()
        ));
    }
    
    Ok(())
}