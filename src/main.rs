// ============================================================================
// AlgoSpeak Compiler — Main Entry Point
// ============================================================================
//
// CLI Usage:
//   algoc build <source.alg>     — compile to .asm
//   algoc run <source.alg>       — compile, assemble, link, execute
//   algoc repl                   — interactive REPL
//   algoc <source.alg>           — backwards compatible (same as build)
//
// Pipeline:
//   Source → Lexer → Parser → AST → Semantic Analysis → CodeGen → .asm
//
// The IR and optimizer modules are available for the pipeline:
//   AST → IR Lowering → Optimizer → (future IR-based codegen)
//
// Performance note:
//   The compiler performs no JVM start-up, no LLVM optimisation passes, and no
//   interpretation.  It reads the source once, walks the AST once for semantic
//   checks, and walks it once more to emit assembly — O(n) in source size.
// ============================================================================

mod token;
mod lexer;
mod ast;
mod parser;
mod semantic;
mod ir;
mod optimizer;
mod codegen;
mod repl;

use std::env;
use std::fs;
use std::process::{self, Command};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "build" => {
            if args.len() < 3 {
                eprintln!("Error: 'build' requires a source file.");
                eprintln!("Usage: algoc build <source.alg>");
                process::exit(1);
            }
            compile_file(&args[2], false);
        }
        "run" => {
            if args.len() < 3 {
                eprintln!("Error: 'run' requires a source file.");
                eprintln!("Usage: algoc run <source.alg>");
                process::exit(1);
            }
            compile_file(&args[2], true);
        }
        "repl" => {
            repl::run_repl();
        }
        _ => {
            // Backwards compatible: treat first arg as source file (build mode)
            if args[1].ends_with(".alg") {
                compile_file(&args[1], false);
            } else {
                eprintln!("Unknown command: '{}'", args[1]);
                print_usage();
                process::exit(1);
            }
        }
    }
}

fn print_usage() {
    eprintln!("AlgoSpeak Compiler v2.0.0");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  algoc build <source.alg>   Compile to assembly (.asm)");
    eprintln!("  algoc run <source.alg>     Compile, assemble, link, and execute");
    eprintln!("  algoc repl                 Start interactive REPL");
    eprintln!("  algoc <source.alg>         Same as 'build'");
}

fn compile_file(source_path: &str, run_after: bool) {
    // ── Read source ─────────────────────────────────────────────────────
    let source = match fs::read_to_string(source_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: cannot read '{}': {}", source_path, e);
            process::exit(1);
        }
    };

    // ── Lex ─────────────────────────────────────────────────────────────
    let mut lex = lexer::Lexer::new(&source);
    let tokens = match lex.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Lexer error: {}", e);
            process::exit(1);
        }
    };

    // ── Parse ───────────────────────────────────────────────────────────
    let mut par = parser::Parser::new(tokens);
    let program = match par.parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parser error: {}", e);
            process::exit(1);
        }
    };

    // ── Semantic analysis ───────────────────────────────────────────────
    let mut sem = semantic::SemanticAnalyzer::new();
    if let Err(e) = sem.analyze(&program) {
        eprintln!("{}", e);
        process::exit(1);
    }

    // ── IR lowering + optimisation (for future use / demonstration) ────
    let ir_lowering = ir::IRLowering::new();
    let ir_instructions = ir_lowering.lower(&program);
    let _optimized_ir = optimizer::optimize(ir_instructions);
    // Note: Currently the assembly is generated directly from the AST.
    // The IR pipeline is available and can be used for analysis/debugging.
    // A future version will generate assembly from the optimised IR.

    // ── Code generation (AST-based) ─────────────────────────────────────
    let mut cg = codegen::CodeGen::new();
    let asm = cg.generate(&program);

    // ── Write output .asm file ──────────────────────────────────────────
    let out_path = source_path.replace(".alg", ".asm");

    if let Err(e) = fs::write(&out_path, &asm) {
        eprintln!("Error: cannot write '{}': {}", out_path, e);
        process::exit(1);
    }

    println!("✓ Compiled '{}' → '{}'", source_path, out_path);

    if run_after {
        // ── Assemble ────────────────────────────────────────────────────
        let obj_path = source_path.replace(".alg", ".o");
        let bin_path = source_path.replace(".alg", "");

        println!("  Assembling...");
        let nasm = Command::new("nasm")
            .args(["-f", "elf64", &out_path, "-o", &obj_path])
            .output();

        match nasm {
            Ok(output) if !output.status.success() => {
                eprintln!("NASM error:\n{}", String::from_utf8_lossy(&output.stderr));
                process::exit(1);
            }
            Err(e) => {
                eprintln!("Failed to run nasm: {}. Is NASM installed?", e);
                process::exit(1);
            }
            _ => {}
        }

        // ── Link ────────────────────────────────────────────────────────
        println!("  Linking...");
        let ld = Command::new("ld")
            .args([&obj_path, "-o", &bin_path])
            .output();

        match ld {
            Ok(output) if !output.status.success() => {
                eprintln!("ld error:\n{}", String::from_utf8_lossy(&output.stderr));
                process::exit(1);
            }
            Err(e) => {
                eprintln!("Failed to run ld: {}", e);
                process::exit(1);
            }
            _ => {}
        }

        // ── Execute ─────────────────────────────────────────────────────
        println!("  Running...\n");
        let run = Command::new(&format!("./{}", bin_path))
            .status();

        match run {
            Ok(status) => {
                if !status.success() {
                    if let Some(code) = status.code() {
                        process::exit(code);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to execute '{}': {}", bin_path, e);
                process::exit(1);
            }
        }

        // Clean up intermediate files
        let _ = fs::remove_file(&obj_path);
        let _ = fs::remove_file(&bin_path);
        let _ = fs::remove_file(&out_path);
    } else {
        println!();
        println!("To assemble and run:");
        println!("  nasm -f elf64 {} -o output.o", out_path);
        println!("  ld output.o -o output");
        println!("  ./output");
    }
}
