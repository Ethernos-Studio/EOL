use thiserror::Error;
use std::fmt;

#[derive(Error, Debug, Clone)]
pub enum EolError {
    #[error("词法错误 [{line}:{column}]: {message}\n  提示: {suggestion}")]
    Lexer { 
        line: usize, 
        column: usize, 
        message: String,
        suggestion: String,
    },
    
    #[error("语法错误 [{line}:{column}]: {message}\n  提示: {suggestion}")]
    Parser { 
        line: usize, 
        column: usize, 
        message: String,
        suggestion: String,
    },
    
    #[error("语义错误 [{line}:{column}]: {message}\n  提示: {suggestion}")]
    Semantic { 
        line: usize, 
        column: usize, 
        message: String,
        suggestion: String,
    },
    
    #[error("代码生成错误: {message}\n  提示: {suggestion}")]
    CodeGen { 
        message: String,
        suggestion: String,
    },
    
    #[error("IO错误: {0}")]
    Io(String),
    
    #[error("LLVM错误: {0}")]
    Llvm(String),
    
    #[error("类型错误 [{line}:{column}]: {message}\n  期望类型: {expected}\n  实际类型: {actual}\n  提示: {suggestion}")]
    TypeMismatch {
        line: usize,
        column: usize,
        message: String,
        expected: String,
        actual: String,
        suggestion: String,
    },
    
    #[error("未定义标识符 [{line}:{column}]: '{name}'\n  提示: {suggestion}")]
    UndefinedIdentifier {
        line: usize,
        column: usize,
        name: String,
        suggestion: String,
    },
    
    #[error("重复定义 [{line}:{column}]: '{name}'\n  提示: {suggestion}")]
    DuplicateDefinition {
        line: usize,
        column: usize,
        name: String,
        suggestion: String,
    },
}

pub type EolResult<T> = Result<T, EolError>;

#[derive(Debug, Clone)]
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
pub fn lexer_error(line: usize, column: usize, message: impl Into<String>) -> EolError {
    let msg = message.into();
    let suggestion = get_lexer_suggestion(&msg);
    EolError::Lexer {
        line,
        column,
        message: msg,
        suggestion,
    }
}

// 语法错误
pub fn parser_error(line: usize, column: usize, message: impl Into<String>) -> EolError {
    let msg = message.into();
    let suggestion = get_parser_suggestion(&msg);
    EolError::Parser {
        line,
        column,
        message: msg,
        suggestion,
    }
}

// 语义错误
pub fn semantic_error(line: usize, column: usize, message: impl Into<String>) -> EolError {
    let msg = message.into();
    let suggestion = get_semantic_suggestion(&msg);
    EolError::Semantic {
        line,
        column,
        message: msg,
        suggestion,
    }
}

// 代码生成错误
pub fn codegen_error(message: impl Into<String>) -> EolError {
    let msg = message.into();
    let suggestion = get_codegen_suggestion(&msg);
    EolError::CodeGen {
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
) -> EolError {
    let expected_str = expected.into();
    let actual_str = actual.into();
    let suggestion = format!("请确保表达式返回 '{}' 类型的值", expected_str);
    EolError::TypeMismatch {
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
) -> EolError {
    let name_str = name.into();
    let suggestion = format!("请检查 '{}' 的拼写，或在使用前声明该变量/函数", name_str);
    EolError::UndefinedIdentifier {
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
) -> EolError {
    let name_str = name.into();
    let suggestion = format!("'{}' 已被定义，请使用不同的名称", name_str);
    EolError::DuplicateDefinition {
        line,
        column,
        name: name_str,
        suggestion,
    }
}

// 根据错误信息提供词法分析建议
fn get_lexer_suggestion(message: &str) -> String {
    if message.contains("Unexpected character") {
        "请检查是否有非法字符，EOL 仅支持标准 ASCII 字符".to_string()
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

// 打印带有上下文的错误信息
pub fn print_error_with_context(error: &EolError, source: &str, filename: &str) {
    eprintln!("\n[编译错误]");
    eprintln!("文件: {}", filename);
    
    // 获取错误位置
    let (line, column) = match error {
        EolError::Lexer { line, column, .. } => (*line, *column),
        EolError::Parser { line, column, .. } => (*line, *column),
        EolError::Semantic { line, column, .. } => (*line, *column),
        EolError::TypeMismatch { line, column, .. } => (*line, *column),
        EolError::UndefinedIdentifier { line, column, .. } => (*line, *column),
        EolError::DuplicateDefinition { line, column, .. } => (*line, *column),
        _ => (0, 0),
    };
    
    if line > 0 {
        eprintln!("位置: 第 {} 行, 第 {} 列", line, column);
        
        // 打印源代码上下文
        let lines: Vec<&str> = source.lines().collect();
        let start = line.saturating_sub(3).max(1);
        let end = (line + 1).min(lines.len());
        
        eprintln!("\n源代码上下文:");
        for i in start..=end {
            if i <= lines.len() {
                eprintln!("{:4} | {}", i, lines[i - 1]);
                if i == line {
                    // 打印错误指示器
                    let spaces = " ".repeat(column.saturating_sub(1) + 6);
                    eprintln!("{}^ 错误发生在这里", spaces);
                }
            }
        }
    }
    
    eprintln!("\n{}", error);
    eprintln!();
}
