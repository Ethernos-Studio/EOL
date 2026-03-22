use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use cavvy::error::print_error_with_context;
use cavvy::lexer;
use cavvy::parser;
use cavvy::preprocessor;
use cavvy::semantic;

/// 获取系统包含路径（caylibs目录）
fn get_system_include_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    // 1. 从可执行文件所在目录查找 caylibs
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let exe_caylibs = exe_dir.join("caylibs");
            if exe_caylibs.exists() {
                paths.push(exe_caylibs);
            }
        }
    }
    
    // 2. 从当前工作目录查找 caylibs
    let cwd_caylibs = PathBuf::from("caylibs");
    if cwd_caylibs.exists() && !paths.contains(&cwd_caylibs) {
        paths.push(cwd_caylibs);
    }
    
    paths
}

const VERSION: &str = env!("CAY_CHECK_VERSION");

fn print_usage() {
    println!("Cay Check v{}", VERSION);
    println!("Usage: cay-check [options] <source_file.cay>");
    println!("");
    println!("Options:");
    println!("  --lex-only            只进行词法分析");
    println!("  --parse-only          进行词法和语法分析（不进行语义分析）");
    println!("  --no-preprocess       跳过预处理阶段");
    println!("  --version, -v         显示版本号");
    println!("  --help, -h            显示帮助信息");
    println!("");
    println!("Examples:");
    println!("  cay-check hello.cay");
    println!("  cay-check --lex-only hello.cay");
    println!("  cay-check --parse-only hello.cay");
}

#[derive(Debug, Clone, Copy)]
enum CheckLevel {
    LexOnly,
    ParseOnly,
    Full,
}

impl Default for CheckLevel {
    fn default() -> Self {
        CheckLevel::Full
    }
}

struct CheckOptions {
    level: CheckLevel,
    preprocess: bool,
}

impl Default for CheckOptions {
    fn default() -> Self {
        CheckOptions {
            level: CheckLevel::default(),
            preprocess: true,
        }
    }
}

fn parse_args(args: &[String]) -> Result<(CheckOptions, String), String> {
    let mut options = CheckOptions::default();
    let mut input_file: Option<String> = None;
    let mut i = 1;

    while i < args.len() {
        let arg = &args[i];

        match arg.as_str() {
            "--version" | "-v" => {
                println!("Cavvy Check v{}", VERSION);
                process::exit(0);
            }
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            "--lex-only" => {
                options.level = CheckLevel::LexOnly;
            }
            "--parse-only" => {
                options.level = CheckLevel::ParseOnly;
            }
            "--no-preprocess" => {
                options.preprocess = false;
            }
            _ => {
                if arg.starts_with('-') {
                    return Err(format!("未知选项: {}", arg));
                }
                if input_file.is_none() {
                    input_file = Some(arg.clone());
                } else {
                    return Err(format!("多余参数: {}", arg));
                }
            }
        }
        i += 1;
    }

    let input_file = input_file.ok_or("需要指定输入文件")?;

    Ok((options, input_file))
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let (options, source_path) = match parse_args(&args) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("错误: {}", e);
            print_usage();
            process::exit(1);
        }
    };

    println!("Cavvy Check v{}", VERSION);
    println!("检查文件: {}", source_path);
    println!("检查级别: {}", match options.level {
        CheckLevel::LexOnly => "词法分析",
        CheckLevel::ParseOnly => "语法分析",
        CheckLevel::Full => "完整检查（预处理+词法+语法+语义）",
    });
    println!("预处理: {}", if options.preprocess { "启用" } else { "跳过" });
    println!("");

    let source = match fs::read_to_string(&source_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("错误读取源文件 '{}': {}", source_path, e);
            process::exit(1);
        }
    };

    let start_time = std::time::Instant::now();

    // 预处理阶段
    let processed_source = if options.preprocess {
        println!("[0] 预处理...");
        let base_dir = Path::new(&source_path)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf()));
        
        // 获取系统包含路径
        let system_paths = get_system_include_paths();
        
        // 使用带系统路径的预处理器
        let base_dir_str = base_dir.to_str().unwrap_or(".");
        let preprocess_result = if system_paths.is_empty() {
            preprocessor::preprocess(&source, &source_path, base_dir_str)
        } else {
            let mut pp = preprocessor::Preprocessor::with_system_paths(base_dir_str, system_paths);
            pp.process(&source, &source_path)
        };
        
        match preprocess_result {
            Ok(processed) => {
                println!("  [+] 预处理通过");
                processed
            }
            Err(e) => {
                print_error_with_context(&e, &source, &source_path);
                process::exit(1);
            }
        }
    } else {
        source
    };

    match options.level {
        CheckLevel::LexOnly => {
            if options.preprocess {
                println!("");
            }
            println!("[1] 词法分析...");
            match lexer::lex(&processed_source) {
                Ok(tokens) => {
                    let elapsed = start_time.elapsed();
                    println!("  [+] 词法分析通过");
                    println!("      发现 {} 个 token", tokens.len());
                    println!("");
                    println!("[+] 语法检查完成! (耗时: {:?})", elapsed);
                }
                Err(e) => {
                    print_error_with_context(&e, &processed_source, &source_path);
                    process::exit(1);
                }
            }
        }
        CheckLevel::ParseOnly => {
            if options.preprocess {
                println!("");
            }
            println!("[1] 词法分析...");
            let tokens = match lexer::lex(&processed_source) {
                Ok(tokens) => {
                    println!("  [+] 词法分析通过");
                    tokens
                }
                Err(e) => {
                    print_error_with_context(&e, &processed_source, &source_path);
                    process::exit(1);
                }
            };

            println!("");
            println!("[2] 语法分析...");
            match parser::parse(tokens) {
                Ok(ast) => {
                    let elapsed = start_time.elapsed();
                    println!("  [+] 语法分析通过");
                    println!("      发现 {} 个类定义", ast.classes.len());
                    println!("");
                    println!("[+] 语法检查完成! (耗时: {:?})", elapsed);
                }
                Err(e) => {
                    print_error_with_context(&e, &processed_source, &source_path);
                    process::exit(1);
                }
            }
        }
        CheckLevel::Full => {
            if options.preprocess {
                println!("");
            }
            println!("[1] 词法分析...");
            let tokens = match lexer::lex(&processed_source) {
                Ok(tokens) => {
                    println!("  [+] 词法分析通过");
                    tokens
                }
                Err(e) => {
                    print_error_with_context(&e, &processed_source, &source_path);
                    process::exit(1);
                }
            };

            println!("");
            println!("[2] 语法分析...");
            let ast = match parser::parse(tokens) {
                Ok(ast) => {
                    println!("  [+] 语法分析通过");
                    ast
                }
                Err(e) => {
                    print_error_with_context(&e, &processed_source, &source_path);
                    process::exit(1);
                }
            };

            println!("");
            println!("[3] 语义分析...");
            let mut analyzer = semantic::SemanticAnalyzer::new();
            match analyzer.analyze(&ast) {
                Ok(_) => {
                    let elapsed = start_time.elapsed();
                    println!("  [+] 语义分析通过");
                    println!("");
                    println!("[+] 语法检查完成! (耗时: {:?})", elapsed);
                }
                Err(e) => {
                    print_error_with_context(&e, &processed_source, &source_path);
                    process::exit(1);
                }
            }
        }
    }
}
