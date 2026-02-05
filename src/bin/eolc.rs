use std::env;
use std::fs;
use std::process;
use std::path::Path;
use eol::Compiler;
use eol::error::{print_error_with_context, EolError};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    println!("eolc v{}", VERSION);
    println!("Usage: eolc <source_file.eol> [output_file.exe]");
    println!("       eolc --version");
    println!("");
    println!("EOL (Ethernos Object Language) to Windows EXE Compiler");
    println!("Compiles .eol source files directly to Windows executable (.exe)");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }
    
    // 处理 --version 参数
    if args[1] == "--version" || args[1] == "-v" {
        println!("eolc v{}", VERSION);
        process::exit(0);
    }
    
    let source_path = &args[1];
    let exe_output = if args.len() >= 3 {
        args[2].clone()
    } else {
        // 默认输出文件名
        Path::new(source_path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(|stem| format!("{}.exe", stem))
            .unwrap_or_else(|| "output.exe".to_string())
    };
    
    // 生成临时的IR文件名
    let ir_file = Path::new(&exe_output)
        .with_extension("ll")
        .to_string_lossy()
        .to_string();
    
    println!("EOL 编译器 v{}", VERSION);
    println!("源文件: {}", source_path);
    println!("输出: {}", exe_output);
    println!("");
    
    // 1. EOL → IR
    println!("[1] EOL → IR 编译...");
    let source = match fs::read_to_string(source_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("错误读取源文件 '{}': {}", source_path, e);
            process::exit(1);
        }
    };
    
    let compiler = Compiler::new();
    match compiler.compile(&source, &ir_file) {
        Ok(_) => {
            println!("  [+] EOL 编译成功");
        }
        Err(e) => {
            // 使用新的错误打印功能显示详细的错误信息
            print_error_with_context(&e, &source, source_path);
            process::exit(1);
        }
    }
    
    // 2. IR → EXE (调用ir2exe)
    println!("");
    println!("[2] IR → EXE 编译...");
    
    // 获取当前执行目录
    let current_exe = env::current_exe().unwrap_or_else(|_| {
        eprintln!("无法获取当前执行路径");
        process::exit(1);
    });
    
    let bin_dir = current_exe.parent().unwrap_or_else(|| {
        eprintln!("无法获取执行目录");
        process::exit(1);
    });
    
    let ir2exe_path = bin_dir.join("ir2exe.exe");
    
    if !ir2exe_path.exists() {
        eprintln!("错误: 找不到 ir2exe.exe 在 {:?}", ir2exe_path);
        eprintln!("请确保 ir2exe.exe 与 eolc.exe 在同一目录");
        // 清理IR文件
        let _ = fs::remove_file(&ir_file);
        process::exit(1);
    }
    
    // 调用ir2exe
    let output = process::Command::new(&ir2exe_path)
        .args(&[&ir_file, &exe_output])
        .output()
        .unwrap_or_else(|e| {
            eprintln!("执行ir2exe失败: {}", e);
            // 清理IR文件
            let _ = fs::remove_file(&ir_file);
            process::exit(1);
        });
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        eprintln!("IR→EXE编译失败");
        if !error_msg.is_empty() {
            eprintln!("错误: {}", error_msg);
        }
        // 清理IR文件
        let _ = fs::remove_file(&ir_file);
        process::exit(1);
    }
    
    // 清理IR文件
    if let Err(e) = fs::remove_file(&ir_file) {
        eprintln!("警告: 无法清理临时文件 {}: {}", ir_file, e);
    }
    
    println!("");
    println!("[+] 编译完成!");
    println!("生成: {}", exe_output);
}
