use thiserror::Error;
use std::fmt;
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::sync::Arc;

#[derive(Error, Debug, Clone)]
pub enum cayError {
    #[error("词法错误 [{line}:{column}]: {message}")]
    Lexer { 
        line: usize, 
        column: usize, 
        message: String,
        suggestion: String,
    },
    
    #[error("语法错误 [{line}:{column}]: {message}")]
    Parser { 
        line: usize, 
        column: usize, 
        message: String,
        suggestion: String,
    },
    
    #[error("语义错误 [{line}:{column}]: {message}")]
    Semantic { 
        line: usize, 
        column: usize, 
        message: String,
        suggestion: String,
    },
    
    #[error("代码生成错误: {message}")]
    CodeGen { 
        message: String,
        suggestion: String,
    },
    
    #[error("IO错误: {0}")]
    Io(String),
    
    #[error("LLVM错误: {0}")]
    Llvm(String),
    
    #[error("类型错误 [{line}:{column}]: {message}")]
    TypeMismatch {
        line: usize,
        column: usize,
        message: String,
        expected: String,
        actual: String,
        suggestion: String,
    },
    
    #[error("未定义标识符 [{line}:{column}]: '{name}'")]
    UndefinedIdentifier {
        line: usize,
        column: usize,
        name: String,
        suggestion: String,
    },
    
    #[error("重复定义 [{line}:{column}]: '{name}'")]
    DuplicateDefinition {
        line: usize,
        column: usize,
        name: String,
        suggestion: String,
    },

    #[error("预处理器错误 [{line}:{column}]: {message}")]
    Preprocessor { 
        line: usize, 
        column: usize, 
        message: String,
        suggestion: String,
    },
}

pub type cayResult<T> = Result<T, cayError>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

// 词法错误
pub fn lexer_error(line: usize, column: usize, message: impl Into<String>) -> cayError {
    let msg = message.into();
    let suggestion = get_lexer_suggestion(&msg);
    cayError::Lexer {
        line,
        column,
        message: msg,
        suggestion,
    }
}

// 语法错误
pub fn parser_error(line: usize, column: usize, message: impl Into<String>) -> cayError {
    let msg = message.into();
    let suggestion = get_parser_suggestion(&msg);
    cayError::Parser {
        line,
        column,
        message: msg,
        suggestion,
    }
}

// 语义错误
pub fn semantic_error(line: usize, column: usize, message: impl Into<String>) -> cayError {
    let msg = message.into();
    let suggestion = get_semantic_suggestion(&msg);
    cayError::Semantic {
        line,
        column,
        message: msg,
        suggestion,
    }
}

// 代码生成错误
pub fn codegen_error(message: impl Into<String>) -> cayError {
    let msg = message.into();
    let suggestion = get_codegen_suggestion(&msg);
    cayError::CodeGen {
        message: msg,
        suggestion,
    }
}

// 类型不匹配错误
pub fn type_mismatch_error(
    line: usize,
    column: usize,
    expected: impl Into<String>,
    actual: impl Into<String>,
) -> cayError {
    let expected_str = expected.into();
    let actual_str = actual.into();
    let suggestion = format!("请确保表达式返回 '{}' 类型的值", expected_str);
    cayError::TypeMismatch {
        line,
        column,
        message: format!("类型不匹配: 期望 '{}', 实际 '{}'", expected_str, actual_str),
        expected: expected_str,
        actual: actual_str,
        suggestion,
    }
}

// 未定义标识符错误
pub fn undefined_identifier_error(
    line: usize,
    column: usize,
    name: impl Into<String>,
) -> cayError {
    let name_str = name.into();
    let suggestion = format!("请检查 '{}' 的拼写，或在使用前声明该变量/函数", name_str);
    cayError::UndefinedIdentifier {
        line,
        column,
        name: name_str,
        suggestion,
    }
}

// 重复定义错误
pub fn duplicate_definition_error(
    line: usize,
    column: usize,
    name: impl Into<String>,
) -> cayError {
    let name_str = name.into();
    let suggestion = format!("'{}' 已被定义，请使用不同的名称", name_str);
    cayError::DuplicateDefinition {
        line,
        column,
        name: name_str,
        suggestion,
    }
}

// 根据错误信息提供词法分析建议
fn get_lexer_suggestion(message: &str) -> String {
    if message.contains("Unexpected character") {
        "请检查是否有非法字符，cay 仅支持标准 ASCII 字符".to_string()
    } else if message.contains("Unterminated string") {
        "字符串字面量必须使用双引号闭合".to_string()
    } else if message.contains("Invalid escape") {
        "转义字符必须是以下之一: \\n \\t \\\" \\\\'. 不支持其他转义序列".to_string()
    } else {
        "请检查代码语法".to_string()
    }
}

// 根据错误信息提供语法分析建议
fn get_parser_suggestion(message: &str) -> String {
    if message.contains("Expected ';'") {
        "语句末尾必须添加分号 (;)".to_string()
    } else if message.contains("Expected '{'") {
        "代码块必须使用大括号 {} 包裹".to_string()
    } else if message.contains("Expected '('") {
        "条件表达式必须使用括号 () 包裹".to_string()
    } else if message.contains("Unexpected token") {
        "请检查语法结构，可能是关键字拼写错误或缺少必要的符号".to_string()
    } else if message.contains("Expected identifier") {
        "此处需要一个标识符（变量名或函数名）".to_string()
    } else if message.contains("Expected type") {
        "变量声明需要指定类型，如: int, long, String, void".to_string()
    } else {
        "请检查代码语法结构".to_string()
    }
}

// 根据错误信息提供语义分析建议
fn get_semantic_suggestion(message: &str) -> String {
    if message.contains("Type mismatch") {
        "类型不匹配。请确保赋值或表达式中的类型一致".to_string()
    } else if message.contains("Undefined variable") {
        "变量未定义。请在使用前声明变量".to_string()
    } else if message.contains("Undefined function") {
        "函数未定义。请检查函数名拼写或声明函数".to_string()
    } else if message.contains("Duplicate") {
        "重复定义。请使用不同的名称".to_string()
    } else if message.contains("main method") {
        "程序必须包含一个 public static void main() 方法作为入口".to_string()
    } else if message.contains("return type") {
        "返回值类型与函数声明不匹配".to_string()
    } else if message.contains("cannot assign") {
        "赋值错误。请确保左侧是可赋值的变量".to_string()
    } else if message.contains("Operator") {
        "运算符不支持这些类型的操作数".to_string()
    } else {
        "请检查语义正确性".to_string()
    }
}

// 根据错误信息提供代码生成建议
fn get_codegen_suggestion(message: &str) -> String {
    if message.contains("Unsupported") {
        "此功能暂不支持。请查看文档了解支持的特性".to_string()
    } else if message.contains("main function") {
        "请确保定义了 public static void main() 方法".to_string()
    } else {
        "代码生成失败，请检查代码结构".to_string()
    }
}

/// 将行号列号转换为字节偏移量
fn line_col_to_offset(source: &str, line: usize, column: usize) -> usize {
    let mut current_line = 1;
    let mut current_col = 1;
    
    for (offset, ch) in source.char_indices() {
        if current_line == line && current_col == column {
            return offset;
        }
        
        if ch == '\n' {
            current_line += 1;
            current_col = 1;
        } else {
            current_col += 1;
        }
    }
    
    source.len()
}

/// 计算错误位置的跨度
fn get_error_span(source: &str, line: usize, column: usize, error: &cayError) -> SourceSpan {
    let offset = line_col_to_offset(source, line, column);
    
    // 根据错误类型确定跨度长度
    let length = match error {
        cayError::UndefinedIdentifier { name, .. } => name.len(),
        cayError::DuplicateDefinition { name, .. } => name.len(),
        cayError::TypeMismatch { .. } => {
            // 尝试找到该位置的token长度
            let rest = &source[offset..];
            rest.split_whitespace().next().map(|s| s.len()).unwrap_or(1)
        }
        _ => 1,
    };
    
    (offset, length).into()
}

/// 获取错误代码
fn get_error_code(error: &cayError) -> &'static str {
    match error {
        cayError::Lexer { .. } => "cavvy::lexer_error",
        cayError::Parser { .. } => "cavvy::parser_error",
        cayError::Semantic { .. } => "cavvy::semantic_error",
        cayError::TypeMismatch { .. } => "cavvy::type_mismatch",
        cayError::UndefinedIdentifier { .. } => "cavvy::undefined_identifier",
        cayError::DuplicateDefinition { .. } => "cavvy::duplicate_definition",
        cayError::CodeGen { .. } => "cavvy::codegen_error",
        cayError::Io(_) => "cavvy::io_error",
        cayError::Llvm(_) => "cavvy::llvm_error",
        cayError::Preprocessor { .. } => "cavvy::preprocessor_error",
    }
}

/// 获取错误消息（不含建议）
fn get_error_message(error: &cayError) -> String {
    match error {
        cayError::Lexer { message, .. } => format!("词法错误: {}", message),
        cayError::Parser { message, .. } => format!("语法错误: {}", message),
        cayError::Semantic { message, .. } => format!("语义错误: {}", message),
        cayError::TypeMismatch { message, .. } => format!("类型不匹配: {}", message),
        cayError::UndefinedIdentifier { name, .. } => format!("未定义标识符: '{}'", name),
        cayError::DuplicateDefinition { name, .. } => format!("重复定义: '{}'", name),
        cayError::CodeGen { message, .. } => format!("代码生成错误: {}", message),
        cayError::Io(msg) => format!("IO错误: {}", msg),
        cayError::Llvm(msg) => format!("LLVM错误: {}", msg),
        cayError::Preprocessor { message, .. } => format!("预处理器错误: {}", message),
    }
}

/// 获取错误帮助信息
fn get_error_help(error: &cayError) -> Option<String> {
    match error {
        cayError::Lexer { suggestion, .. } => Some(suggestion.clone()),
        cayError::Parser { suggestion, .. } => Some(suggestion.clone()),
        cayError::Semantic { suggestion, .. } => Some(suggestion.clone()),
        cayError::TypeMismatch { suggestion, .. } => Some(suggestion.clone()),
        cayError::UndefinedIdentifier { suggestion, .. } => Some(suggestion.clone()),
        cayError::DuplicateDefinition { suggestion, .. } => Some(suggestion.clone()),
        cayError::CodeGen { suggestion, .. } => Some(suggestion.clone()),
        cayError::Io(_) => None,
        cayError::Llvm(_) => None,
        cayError::Preprocessor { suggestion, .. } => Some(suggestion.clone()),
    }
}

/// 获取错误位置
fn get_error_location(error: &cayError) -> Option<(usize, usize)> {
    match error {
        cayError::Lexer { line, column, .. } => Some((*line, *column)),
        cayError::Parser { line, column, .. } => Some((*line, *column)),
        cayError::Semantic { line, column, .. } => Some((*line, *column)),
        cayError::TypeMismatch { line, column, .. } => Some((*line, *column)),
        cayError::UndefinedIdentifier { line, column, .. } => Some((*line, *column)),
        cayError::DuplicateDefinition { line, column, .. } => Some((*line, *column)),
        cayError::Preprocessor { line, column, .. } => Some((*line, *column)),
        _ => None,
    }
}

/// 打印带有上下文的错误信息 - 使用miette格式
/// 
/// # Arguments
/// * `error` - 错误对象
/// * `source` - 源代码内容
/// * `filename` - 源文件名
/// 
/// # Example
/// ```
/// use cavvy::error::{lexer_error, print_error_with_context};
/// let error = lexer_error(1, 1, "无效的字符");
/// print_error_with_context(&error, "let x = @", "test.cay");
/// ```
pub fn print_error_with_context(error: &cayError, source: &str, filename: &str) {
    // 尝试获取错误位置
    if let Some((line, column)) = get_error_location(error) {
        if line > 0 {
            print_error_with_location(error, source, filename, line, column);
            return;
        }
    }
    
    // 没有位置信息的错误
    print_error_without_location(error);
}

/// 打印带有位置信息的错误
fn print_error_with_location(
    error: &cayError,
    source: &str,
    filename: &str,
    line: usize,
    column: usize,
) {
    let code = get_error_code(error);
    let message = get_error_message(error);
    let help = get_error_help(error);
    
    // 使用 miette 风格格式
    eprintln!("\n  × {}: {}", code, message);
    eprintln!("   ╭─[{}:{}:{}]", filename, line, column);
    
    // 打印源代码上下文（前后3行）
    let lines: Vec<&str> = source.lines().collect();
    let start_line = line.saturating_sub(3).max(1);
    let end_line = (line + 2).min(lines.len());
    
    for i in start_line..=end_line {
        if i <= lines.len() {
            let line_content = lines[i - 1];
            eprintln!("{:3} │ {}", i, line_content);
            
            if i == line {
                // 打印错误指示器
                let prefix_len = column.saturating_sub(1);
                let span_len = get_highlight_length(error);
                let spaces = " ".repeat(prefix_len);
                let arrows = "─".repeat(span_len.max(1));
                eprintln!("    │ {}{} {}", spaces, arrows, "错误在这里");
            }
        }
    }
    
    eprintln!("   ╰────");
    
    // 打印帮助信息
    if let Some(help_text) = help {
        if !help_text.is_empty() {
            eprintln!("  help: {}", help_text);
        }
    }
    
    eprintln!();
}

/// 打印没有位置信息的错误
fn print_error_without_location(error: &cayError) {
    let code = get_error_code(error);
    let message = get_error_message(error);
    let help = get_error_help(error);
    
    eprintln!("\n  × {}: {}", code, message);
    
    if let Some(help_text) = help {
        if !help_text.is_empty() {
            eprintln!("  help: {}", help_text);
        }
    }
    
    eprintln!();
}

/// 获取高亮长度
fn get_highlight_length(error: &cayError) -> usize {
    match error {
        cayError::UndefinedIdentifier { name, .. } => name.len(),
        cayError::DuplicateDefinition { name, .. } => name.len(),
        _ => 1,
    }
}

/// 通用错误打印函数 - 用于非编译错误（如IO错误、配置错误等）
/// 
/// # Arguments
/// * `error_type` - 错误类型标识
/// * `message` - 错误消息
/// * `help` - 可选的帮助信息
/// 
/// # Example
/// ```
/// use cavvy::error::print_miette_error;
/// print_miette_error("cavvy::io_error", "无法读取文件", Some("请检查文件路径是否正确"));
/// ```
pub fn print_miette_error(error_type: &str, message: &str, help: Option<&str>) {
    eprintln!("\n  × {}: {}", error_type, message);
    
    if let Some(help_text) = help {
        if !help_text.is_empty() {
            eprintln!("  help: {}", help_text);
        }
    }
    
    eprintln!();
}

/// 编译阶段错误打印函数
/// 
/// # Arguments
/// * `stage` - 编译阶段（如 "词法分析", "语法分析" 等）
/// * `error` - 错误消息
/// * `source_path` - 源文件路径
/// * `help` - 可选的帮助信息
/// 
/// # Example
/// ```
/// use cavvy::error::print_compile_error;
/// print_compile_error("词法分析", "无效的字符", "test.cay", Some("请检查字符编码"));
/// ```
pub fn print_compile_error(stage: &str, error: &str, source_path: &str, help: Option<&str>) {
    eprintln!("\n  × cavvy::compile_error: {}阶段错误", stage);
    eprintln!("   ╭─[{}]", source_path);
    eprintln!("   │");
    eprintln!("   │ {}", error);
    eprintln!("   ╰────");
    
    if let Some(help_text) = help {
        if !help_text.is_empty() {
            eprintln!("  help: {}", help_text);
        }
    }
    
    eprintln!();
}

/// 外部工具错误打印函数
/// 
/// # Arguments
/// * `tool` - 工具名称（如 "clang", "ir2exe" 等）
/// * `message` - 错误消息
/// * `help` - 可选的帮助信息
/// 
/// # Example
/// ```
/// use cavvy::error::print_tool_error;
/// print_tool_error("clang", "编译失败", Some("请检查 LLVM 安装"));
/// ```
pub fn print_tool_error(tool: &str, message: &str, help: Option<&str>) {
    eprintln!("\n  × cavvy::tool_error: {} 执行失败", tool);
    eprintln!("   │");
    eprintln!("   │ {}", message);
    
    if let Some(help_text) = help {
        if !help_text.is_empty() {
            eprintln!("   │");
            eprintln!("  help: {}", help_text);
        }
    }
    
    eprintln!();
}

/// 警告信息打印函数
/// 
/// # Arguments
/// * `message` - 警告消息
/// 
/// # Example
/// ```
/// use cavvy::error::print_warning;
/// print_warning("未使用的变量 'x'");
/// ```
pub fn print_warning(message: &str) {
    eprintln!("  ⚠ cavvy::warning: {}", message);
}

/// 警告信息打印函数（带位置）
/// 
/// # Arguments
/// * `message` - 警告消息
/// * `filename` - 文件名
/// * `line` - 行号
/// * `column` - 列号
/// 
/// # Example
/// ```
/// use cavvy::error::print_warning_with_location;
/// print_warning_with_location("未使用的变量", "test.cay", 10, 5);
/// ```
pub fn print_warning_with_location(message: &str, filename: &str, line: usize, column: usize) {
    eprintln!("  ⚠ cavvy::warning: {}", message);
    eprintln!("     位置: {}:{}:{}", filename, line, column);
}
