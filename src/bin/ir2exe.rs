use std::env;
use std::process;
use std::path::Path;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    println!("ir2exe v{}", VERSION);
    println!("Usage: ir2exe <input_file.ll> [output_file.exe]");
    println!("       ir2exe --version");
    println!("");
    println!("LLVM IR to Windows EXE Compiler (MinGW-w64 mode)");
    println!("Compiles .ll IR files to Windows executable (.exe) using MinGW-w64");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }
    
    // 处理 --version 参数
    if args[1] == "--version" || args[1] == "-v" {
        println!("ir2exe v{}", VERSION);
        process::exit(0);
    }
    
    let input_file = &args[1];
    let output_file = if args.len() >= 3 {
        args[2].clone()
    } else {
        Path::new(input_file)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(|stem| format!("{}.exe", stem))
            .unwrap_or_else(|| "output.exe".to_string())
    };
    
    println!("IR 编译器 v{} (MinGW-w64 模式)", VERSION);
    println!("IR 文件: {}", input_file);
    println!("输出: {}", output_file);
    println!("");
    
    let current_dir = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    
    let clang_exe = current_dir.join("llvm-minimal/bin/clang.exe");
    let ld_exe = current_dir.join("mingw-minimal/bin/ld.exe");
    
    if !clang_exe.exists() {
        eprintln!("错误: 找不到 clang.exe 在 {:?}", clang_exe);
        process::exit(1);
    }
    
    if !ld_exe.exists() {
        eprintln!("错误: 找不到 ld.exe 在 {:?}", ld_exe);
        process::exit(1);
    }
    
    println!("[I] 正在编译 IR → EXE...");
    
    // 设置库路径
    let lib_path1 = current_dir.join("lib/mingw64/x86_64-w64-mingw32/lib");
    let lib_path2 = current_dir.join("lib/mingw64/lib");
    let lib_path3 = current_dir.join("lib/mingw64/lib/gcc/x86_64-w64-mingw32/15.2.0");
    
    // 使用 -B 指定链接器，使用 -Wl 传递选项给链接器
    let output = process::Command::new(&clang_exe)
        .arg(input_file)
        .arg("-o").arg(&output_file)
        .arg("-target").arg("x86_64-w64-mingw32")
        .arg("-O2")
        .arg("-Wno-override-module")
        .arg("-fuse-ld=lld")  // 使用 LLVM 的 lld 链接器，它更兼容
        .arg("-L").arg(&lib_path1)
        .arg("-L").arg(&lib_path2)
        .arg("-L").arg(&lib_path3)
        .arg("-lkernel32")
        .arg("-lmsvcrt")
        .arg("-ladvapi32")
        .output()
        .unwrap_or_else(|e| {
            eprintln!("执行clang失败: {}", e);
            process::exit(1);
        });
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        eprintln!("编译失败 (clang exit code: {})", output.status.code().unwrap_or(-1));
        eprintln!("错误: {}", error_msg);
        process::exit(1);
    }
    
    if !output.stderr.is_empty() {
        let warn_msg = String::from_utf8_lossy(&output.stderr);
        println!("  [W] {}", warn_msg);
    }
    
    let exe_size = std::fs::metadata(&output_file)
        .map(|m| m.len() as f64 / 1024.0)
        .unwrap_or(0.0);
    println!("  [+] 生成: {} ({:.1} KB)", output_file, exe_size);
    
    println!("");
    println!("[I] 提示: 使用 '{}' 可直接运行并测速", output_file);
    println!("");
    println!("编译完成 (MinGW-w64 模式)");
}
