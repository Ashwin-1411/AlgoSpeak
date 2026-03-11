// ============================================================================
// AlgoSpeak Compiler — REPL (Read-Eval-Print Loop)
// ============================================================================
// Interactive mode for testing AlgoSpeak snippets. Each block of code is:
//   1. Lexed, parsed, semantically checked
//   2. Compiled to assembly
//   3. Assembled with NASM, linked with ld
//   4. Executed, with output displayed to the user
//
// Usage: algospeak repl
// ============================================================================

use std::io::{self, Write};
use std::fs;
use std::process::Command;

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::semantic::SemanticAnalyzer;
use crate::codegen::CodeGen;

pub fn run_repl() {
    println!("AlgoSpeak REPL v2.0.0");
    println!("Type AlgoSpeak code, then press Enter twice (blank line) to execute.");
    println!("Type 'exit' or 'quit' to leave.\n");

    let mut accumulated_source = String::new();

    loop {
        print!("algospeak > ");
        io::stdout().flush().unwrap();

        let mut block = String::new();
        let mut empty_count = 0;

        // Read lines until a blank line or EOF
        loop {
            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Ok(0) => {
                    // EOF
                    println!();
                    return;
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    return;
                }
            }

            let trimmed = line.trim();

            // Check for exit commands
            if trimmed == "exit" || trimmed == "quit" {
                println!("Goodbye!");
                return;
            }

            if trimmed.is_empty() {
                empty_count += 1;
                if empty_count >= 1 && !block.is_empty() {
                    break;
                }
                continue;
            }

            empty_count = 0;
            block.push_str(&line);
            print!("        ... ");
            io::stdout().flush().unwrap();
        }

        if block.trim().is_empty() {
            continue;
        }

        // Accumulate code for context (variables persist across invocations)
        // For simplicity, we compile each block independently
        let source = block.clone();

        // ── Compile ────────────────────────────────────────────────────
        let mut lex = Lexer::new(&source);
        let tokens = match lex.tokenize() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Lexer error: {}", e);
                continue;
            }
        };

        let mut parser = Parser::new(tokens);
        let program = match parser.parse() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Parser error: {}", e);
                continue;
            }
        };

        let mut sem = SemanticAnalyzer::new();
        if let Err(e) = sem.analyze(&program) {
            eprintln!("{}", e);
            continue;
        }

        let mut cg = CodeGen::new();
        let asm = cg.generate(&program);

        // ── Write temp assembly file ───────────────────────────────────
        let tmp_dir = std::env::temp_dir();
        let asm_path = tmp_dir.join("algospeak_repl.asm");
        let obj_path = tmp_dir.join("algospeak_repl.o");
        let bin_path = tmp_dir.join("algospeak_repl");

        if let Err(e) = fs::write(&asm_path, &asm) {
            eprintln!("Error writing temp file: {}", e);
            continue;
        }

        // ── Assemble ──────────────────────────────────────────────────
        let nasm_result = Command::new("nasm")
            .args(["-f", "elf64", asm_path.to_str().unwrap(), "-o", obj_path.to_str().unwrap()])
            .output();

        match nasm_result {
            Ok(output) if !output.status.success() => {
                eprintln!("Assembly error:\n{}", String::from_utf8_lossy(&output.stderr));
                continue;
            }
            Err(e) => {
                eprintln!("Failed to run nasm: {}. Is NASM installed?", e);
                continue;
            }
            _ => {}
        }

        // ── Link ──────────────────────────────────────────────────────
        let ld_result = Command::new("ld")
            .args([obj_path.to_str().unwrap(), "-o", bin_path.to_str().unwrap()])
            .output();

        match ld_result {
            Ok(output) if !output.status.success() => {
                eprintln!("Link error:\n{}", String::from_utf8_lossy(&output.stderr));
                continue;
            }
            Err(e) => {
                eprintln!("Failed to run ld: {}", e);
                continue;
            }
            _ => {}
        }

        // ── Execute ───────────────────────────────────────────────────
        let run_result = Command::new(bin_path.to_str().unwrap())
            .output();

        match run_result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stdout.is_empty() {
                    print!("{}", stdout);
                }
                if !stderr.is_empty() {
                    eprint!("{}", stderr);
                }
            }
            Err(e) => {
                eprintln!("Execution error: {}", e);
            }
        }

        // Clean up temp files
        let _ = fs::remove_file(&asm_path);
        let _ = fs::remove_file(&obj_path);
        let _ = fs::remove_file(&bin_path);

        accumulated_source.push_str(&source);
        accumulated_source.push('\n');
        println!();
    }
}
