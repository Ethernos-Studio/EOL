use std::env;
use std::fs;
use std::process;
use eol::Compiler;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    println!("eolll v{}", VERSION);
    println!("Usage: eolll <source_file.eol> [output_file.ll]");
    println!("       eolll --version");
    println!("");
    println!("EOL (Ethernos Object Language) to LLVM IR Compiler");
    println!("Compiles .eol source files to LLVM IR (.ll)");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }
    
    // 处理 --version 参数
    if args[1] == "--version" || args[1] == "-v" {
        println!("eolll v{}", VERSION);
        process::exit(0);
    }
    
    let source_path = &args[1];
    let output_path = if args.len() >= 3 {
        args[2].clone()
    } else {
        // 默认输出文件名
        if source_path.ends_with(".eol") {
            source_path.replace(".eol", ".ll")
        } else {
            format!("{}.ll", source_path)
        }
    };
    
    // 读取源文件
    let source = match fs::read_to_string(source_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading source file '{}': {}", source_path, e);
            process::exit(1);
        }
    };
    
    println!("eolll v{}", VERSION);
    println!("Compiling: {}", source_path);
    println!("Output: {}", output_path);
    println!("");
    
    // 编译
    let compiler = Compiler::new();
    match compiler.compile(&source, &output_path) {
        Ok(_) => {
            println!("");
            println!("Compilation successful!");
            println!("Generated: {}", output_path);
        }
        Err(e) => {
            eprintln!("Compilation error: {}", e);
            process::exit(1);
        }
    }
}