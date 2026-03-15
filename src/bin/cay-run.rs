use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const VERSION: &str = env!("CAY_RUN_VERSION");

/// 运行时配置
struct RuntimeOptions {
    keep_cache: bool,           // --keep-cache: 保留编译缓存
    cache_dir: Option<String>,  // --cache-dir: 指定缓存目录
    target: String,             // --target: 目标平台
    verbose: bool,              // --verbose: 详细输出
    args: Vec<String>,          // 传递给目标程序的参数
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        RuntimeOptions {
            keep_cache: false,
            cache_dir: None,
            target: get_default_target(),
            verbose: false,
            args: Vec::new(),
        }
    }
}

/// 获取默认目标平台
fn get_default_target() -> String {
    if cfg!(target_os = "windows") {
        "x86_64-w64-mingw32".to_string()
    } else if cfg!(target_os = "linux") {
        "x86_64-unknown-linux-gnu".to_string()
    } else if cfg!(target_os = "macos") {
        "x86_64-apple-darwin".to_string()
    } else {
        "x86_64-unknown-linux-gnu".to_string()
    }
}

/// 获取默认缓存目录
fn get_default_cache_dir() -> PathBuf {
    let temp_dir = env::temp_dir();
    temp_dir.join("cavvy-run-cache")
}

/// 生成唯一的缓存子目录名
fn generate_cache_subdir(source_file: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    let file_hash = compute_simple_hash(source_file);
    format!("cavvy_{}_{}", timestamp, file_hash)
}

/// 计算简单哈希值
fn compute_simple_hash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

fn print_usage() {
    println!("Cavvy Runtime v{}", VERSION);
    println!("Usage: cay-run [options] <source_file.cay> [-- <program_args>]");
    println!("");
    println!("Options:");
    println!("  --keep-cache          保留编译缓存（默认删除）");
    println!("  --cache-dir <path>    指定缓存目录（默认: 系统临时目录/cavvy-run-cache）");
    println!("  --target <target>     指定目标平台");
    println!("  --verbose, -v         显示详细编译信息");
    println!("  --version, -V         显示版本号");
    println!("  --help, -h            显示帮助信息");
    println!("");
    println!("Examples:");
    println!("  cay-run hello.cay");
    println!("  cay-run hello.cay -- arg1 arg2");
    println!("  cay-run --verbose hello.cay");
    println!("  cay-run --keep-cache --cache-dir ./cache hello.cay");
}

fn parse_args(args: &[String]) -> Result<(RuntimeOptions, String), String> {
    let mut options = RuntimeOptions::default();
    let mut source_file: Option<String> = None;
    let mut i = 1;
    let mut found_double_dash = false;

    while i < args.len() {
        let arg = &args[i];

        if arg == "--" {
            found_double_dash = true;
            i += 1;
            // 剩余的所有参数都传递给目标程序
            while i < args.len() {
                options.args.push(args[i].clone());
                i += 1;
            }
            break;
        }

        match arg.as_str() {
            "--version" | "-V" => {
                println!("Cavvy Runtime v{}", VERSION);
                process::exit(0);
            }
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            "--keep-cache" => {
                options.keep_cache = true;
            }
            "--verbose" | "-v" => {
                options.verbose = true;
            }
            "--cache-dir" => {
                if i + 1 < args.len() {
                    options.cache_dir = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    return Err("--cache-dir 需要一个参数".to_string());
                }
            }
            "--target" => {
                if i + 1 < args.len() {
                    options.target = args[i + 1].clone();
                    i += 1;
                } else {
                    return Err("--target 需要一个参数".to_string());
                }
            }
            _ => {
                if arg.starts_with('-') {
                    return Err(format!("未知选项: {}", arg));
                }
                if source_file.is_none() {
                    source_file = Some(arg.clone());
                } else if !found_double_dash {
                    // 如果已经指定了源文件，且还没遇到 --，则报错
                    return Err(format!("多余参数: {}", arg));
                } else {
                    options.args.push(arg.clone());
                }
            }
        }
        i += 1;
    }

    let source_file = source_file.ok_or("需要指定源文件")?;
    Ok((options, source_file))
}

/// 查找 cayc 编译器
fn find_cayc() -> Result<PathBuf, String> {
    // 1. 首先尝试当前目录
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let cayc_paths = [
                exe_dir.join("cayc"),
                exe_dir.join("cayc.exe"),
            ];
            for path in &cayc_paths {
                if path.exists() {
                    return Ok(path.clone());
                }
            }
        }
    }
    
    // 2. 尝试 PATH 中的 cayc
    if let Ok(output) = Command::new("cayc").arg("--version").output() {
        if output.status.success() {
            return Ok(PathBuf::from("cayc"));
        }
    }
    
    Err("找不到 cayc 编译器。请确保 cayc 在 PATH 中或与 cay-run 在同一目录。".to_string())
}

/// 编译 Cavvy 源文件
fn compile_source(
    source_file: &str,
    output_exe: &str,
    options: &RuntimeOptions,
) -> Result<(), String> {
    let cayc_path = find_cayc()?;
    
    if options.verbose {
        println!("[编译] 使用编译器: {}", cayc_path.display());
        println!("[编译] 源文件: {}", source_file);
        println!("[编译] 输出: {}", output_exe);
    }
    
    let mut cmd = Command::new(&cayc_path);
    cmd.arg("-O2")
        .arg("--target")
        .arg(&options.target)
        .arg(source_file)
        .arg(output_exe);
    
    if options.verbose {
        cmd.arg("--keep-ir");
    }
    
    let output = cmd.output()
        .map_err(|e| format!("执行 cayc 失败: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("编译失败:\n{}", stderr));
    }
    
    if options.verbose {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            println!("{}", stdout);
        }
    }
    
    Ok(())
}

/// 运行编译后的可执行文件
fn run_executable(exe_path: &str, args: &[String], verbose: bool) -> Result<i32, String> {
    if verbose {
        println!("[运行] 可执行文件: {}", exe_path);
        if !args.is_empty() {
            println!("[运行] 参数: {:?}", args);
        }
        println!("");
    }
    
    let mut cmd = Command::new(exe_path);
    cmd.args(args);
    
    // 继承标准输入输出
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    
    let status = cmd.status()
        .map_err(|e| format!("运行可执行文件失败: {}", e))?;
    
    Ok(status.code().unwrap_or(-1))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let (options, source_file) = match parse_args(&args) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("错误: {}", e);
            print_usage();
            process::exit(1);
        }
    };
    
    // 检查源文件是否存在
    if !Path::new(&source_file).exists() {
        eprintln!("错误: 源文件 '{}' 不存在", source_file);
        process::exit(1);
    }
    
    // 确定缓存目录
    let cache_base = options.cache_dir.as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(get_default_cache_dir);
    
    let cache_subdir = generate_cache_subdir(&source_file);
    let cache_dir = cache_base.join(&cache_subdir);
    
    if options.verbose {
        println!("Cavvy Runtime v{}", VERSION);
        println!("[缓存] 目录: {}", cache_dir.display());
        println!("");
    }
    
    // 创建缓存目录
    if let Err(e) = fs::create_dir_all(&cache_dir) {
        eprintln!("错误: 无法创建缓存目录 '{}': {}", cache_dir.display(), e);
        process::exit(1);
    }
    
    // 确定可执行文件路径
    let exe_name = if options.target.contains("windows") || options.target.contains("mingw") {
        "program.exe"
    } else {
        "program"
    };
    let exe_path = cache_dir.join(exe_name);
    let exe_path_str = exe_path.to_string_lossy().to_string();
    
    // 编译源文件
    if options.verbose {
        println!("=== 编译阶段 ===");
    }
    
    if let Err(e) = compile_source(&source_file, &exe_path_str, &options) {
        // 清理缓存目录
        if !options.keep_cache {
            let _ = fs::remove_dir_all(&cache_dir);
        }
        eprintln!("{}", e);
        process::exit(1);
    }
    
    if options.verbose {
        println!("");
        println!("=== 运行阶段 ===");
    }
    
    // 运行可执行文件
    let exit_code = match run_executable(&exe_path_str, &options.args, options.verbose) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("错误: {}", e);
            // 清理
            if !options.keep_cache {
                let _ = fs::remove_dir_all(&cache_dir);
            }
            process::exit(1);
        }
    };
    
    // 清理缓存（如果需要）
    if !options.keep_cache {
        if options.verbose {
            println!("");
            println!("[清理] 删除缓存目录: {}", cache_dir.display());
        }
        if let Err(e) = fs::remove_dir_all(&cache_dir) {
            if options.verbose {
                eprintln!("[警告] 无法删除缓存目录: {}", e);
            }
        }
    } else if options.verbose {
        println!("");
        println!("[缓存] 保留缓存目录: {}", cache_dir.display());
    }
    
    process::exit(exit_code);
}
