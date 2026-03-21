use logos::Logos;
use crate::error::{cayResult, lexer_error};
use crate::error::SourceLocation;
use crate::diagnostic::{Diagnostic, DiagnosticCollector, ErrorCodes, CompilationPhase, SourceSpan, FixSuggestion};

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\f]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*([^*]|\*[^/])*\*/")]
pub enum Token {
    // 关键字
    #[token("public")]
    Public,
    #[token("private")]
    Private,
    #[token("protected")]
    Protected,
    #[token("static")]
    Static,
    #[token("final")]
    Final,
    #[token("abstract")]
    Abstract,
    #[token("native")]
    Native,
    // 注解 - 注意：@main 和 @Override 是完整的令牌，不是 @ + 标识符
    #[token("@main")]
    AtMain,
    #[token("@Override")]
    AtOverride,
    #[token("class")]
    Class,
    #[token("void")]
    Void,
    #[token("int")]
    Int,
    #[token("long")]
    Long,
    #[token("float")]
    Float,
    #[token("double")]
    Double,
    #[token("bool")]
    #[token("boolean")]
    Bool,
    #[token("string")]
    #[token("String")]
    String,
    #[token("char")]
    Char,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("do")]
    Do,
    #[token("switch")]
    Switch,
    #[token("case")]
    Case,
    #[token("default")]
    Default,
    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("new")]
    New,
    #[token("this")]
    This,
    #[token("super")]
    Super,
    #[token("extends")]
    Extends,
    #[token("implements")]
    Implements,
    #[token("interface")]
    Interface,
    #[token("instanceof")]
    InstanceOf,
    #[token("var")]
    Var,
    #[token("let")]
    Let,
    #[token("auto")]
    Auto,
    #[token("extern")]
    Extern,

    // FFI 类型关键字
    #[token("c_int")]
    CInt,
    #[token("c_uint")]
    CUInt,
    #[token("c_long")]
    CLong,
    #[token("c_short")]
    CShort,
    #[token("c_ushort")]
    CUShort,
    #[token("c_char")]
    CChar,
    #[token("c_uchar")]
    CUChar,
    #[token("c_float")]
    CFloat,
    #[token("c_double")]
    CDouble,
    #[token("size_t")]
    SizeT,
    #[token("ssize_t")]
    SSizeT,
    #[token("uintptr_t")]
    UIntPtr,
    #[token("intptr_t")]
    IntPtr,
    #[token("c_void")]
    CVoid,
    #[token("c_bool")]
    CBool,

    // 调用约定关键字
    #[token("cdecl")]
    Cdecl,
    #[token("stdcall")]
    Stdcall,
    #[token("fastcall")]
    Fastcall,
    #[token("sysv64")]
    Sysv64,
    #[token("win64")]
    Win64,

    // 标识符
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    
    // 字面量
    #[regex(r"(?:0[xX][0-9a-fA-F][0-9a-fA-F_]*|0[bB][01][01_]*|0[oO]?[0-7][0-7_]*|[0-9][0-9_]*)[Ll]?", |lex| {
        let slice = lex.slice();
        // 分离后缀
        let (num_str, suffix) = if slice.ends_with('L') || slice.ends_with('l') {
            (&slice[..slice.len()-1], Some(slice.chars().last().unwrap()))
        } else {
            (slice, None)
        };
        // 移除下划线
        let cleaned: String = num_str.chars().filter(|c| *c != '_').collect();
        // 解析数字
        let radix = if cleaned.starts_with("0x") || cleaned.starts_with("0X") {
            16
        } else if cleaned.starts_with("0b") || cleaned.starts_with("0B") {
            2
        } else if cleaned.starts_with("0o") || cleaned.starts_with("0O") {
            8
        } else if cleaned.starts_with("0") && cleaned.len() > 1 && cleaned.chars().nth(1).map(|c| c.is_digit(10)).unwrap_or(false) {
            // 以0开头但不含字母的十进制数字？实际上，前导零的十进制数字，但我们将视为十进制（如Java中，前导零表示八进制？在Java中，前导零表示八进制，但为了兼容性，我们将其视为八进制？我们已匹配八进制模式，所以这里应该是十进制）
            10
        } else {
            10
        };
        let num = if radix == 10 {
            cleaned.parse::<i64>().ok()
        } else {
            i64::from_str_radix(&cleaned[2..], radix).ok()
        };
        num.map(|val| (val, suffix))
    })]
    IntegerLiteral(Option<(i64, Option<char>)>),
    
    #[regex(r"(?:[0-9][0-9_]*\.[0-9][0-9_]*|\.[0-9][0-9_]*|[0-9][0-9_]*\.)(?:[eE][+-]?[0-9][0-9_]*)?[FfDd]?", |lex| {
        let slice = lex.slice();
        let (num_str, suffix) = if slice.ends_with('F') || slice.ends_with('f') {
            (&slice[..slice.len()-1], Some('f'))
        } else if slice.ends_with('D') || slice.ends_with('d') {
            (&slice[..slice.len()-1], Some('d'))
        } else {
            (slice, None)
        };
        // 移除下划线
        let cleaned: String = num_str.chars().filter(|c| *c != '_').collect();
        cleaned.parse::<f64>().ok().map(|val| (val, suffix))
    })]
    FloatLiteral(Option<(f64, Option<char>)>),
    
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        let content = &s[1..s.len()-1];
        Some(process_escape_sequences(content))
    })]
    StringLiteral(Option<String>),
    
    #[regex(r"'([^'\\]|\\.)'", |lex| {
        let s = lex.slice();
        let content = &s[1..s.len()-1];
        process_char_escape(content)
    })]
    CharLiteral(Option<char>),
    
    // 运算符
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token("<")]
    Lt,
    #[token("<=")]
    Le,
    #[token(">")]
    Gt,
    #[token(">=")]
    Ge,
    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,
    #[token("!")]
    Bang,
    #[token("&")]
    Ampersand,
    #[token("|")]
    Pipe,
    #[token("^")]
    Caret,
    #[token("<<")]
    Shl,
    #[token(">>")]
    Shr,
    #[token(">>>")]
    UnsignedShr,
    #[token("~")]
    Tilde,
    
    // 赋值运算符
    #[token("=")]
    Assign,
    #[token("+=")]
    AddAssign,
    #[token("-=")]
    SubAssign,
    #[token("*=")]
    MulAssign,
    #[token("/=")]
    DivAssign,
    #[token("%=")]
    ModAssign,
    
    // 自增自减
    #[token("++")]
    Inc,
    #[token("--")]
    Dec,
    
    // 分隔符
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("...")]
    DotDotDot,
    #[token(":")]
    Colon,
    #[token("::")]
    DoubleColon,
    #[token("->")]
    Arrow,
    #[token("?")]
    Question,

    // 换行（用于跟踪行号）- 支持 Windows \r\n 和 Unix \n
    #[regex(r"\r?\n")]
    Newline,
}

#[derive(Debug, Clone)]
pub struct TokenWithLocation {
    pub token: Token,
    pub loc: SourceLocation,
    /// 原始源文件路径（用于支持#include后的错误定位）
    pub source_file: Option<String>,
    /// 原始源文件行号
    pub source_line: Option<usize>,
}

impl TokenWithLocation {
    /// 创建带源映射的token
    pub fn with_source(token: Token, loc: SourceLocation, file: Option<String>, line: Option<usize>) -> Self {
        Self {
            token,
            loc,
            source_file: file,
            source_line: line,
        }
    }

    /// 获取用于错误报告的文件路径
    pub fn get_file(&self) -> Option<&str> {
        self.source_file.as_deref()
    }

    /// 获取用于错误报告的行号
    pub fn get_line(&self) -> usize {
        self.source_line.unwrap_or(self.loc.line)
    }
}

/// 词法分析错误类型
#[derive(Debug, Clone)]
pub enum LexerErrorType {
    InvalidCharacter,
    UnterminatedString,
    InvalidEscapeSequence,
    InvalidNumberLiteral,
    UnterminatedComment,
    InvalidIdentifier,
}

/// 增强的词法分析器，支持诊断收集和源映射
pub struct Lexer<'a> {
    source: &'a str,
    inner: logos::Lexer<'a, Token>,
    line: usize,
    column: usize,
    diagnostics: DiagnosticCollector,
    collect_all_errors: bool,
    /// 当前源文件路径（用于#include后的错误定位）
    current_source_file: Option<String>,
    /// 源映射表：输出行号 -> (原始文件, 原始行号)
    source_map: std::collections::HashMap<usize, (String, usize)>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            inner: Token::lexer(source),
            line: 1,
            column: 1,
            diagnostics: DiagnosticCollector::new(),
            collect_all_errors: false,
            current_source_file: None,
            source_map: std::collections::HashMap::new(),
        }
    }

    /// 创建带源映射的词法分析器
    pub fn with_source_map(source: &'a str, source_map: std::collections::HashMap<usize, (String, usize)>) -> Self {
        Self {
            source,
            inner: Token::lexer(source),
            line: 1,
            column: 1,
            diagnostics: DiagnosticCollector::new(),
            collect_all_errors: false,
            current_source_file: None,
            source_map,
        }
    }

    /// 启用多错误收集模式
    pub fn with_collect_all_errors(mut self) -> Self {
        self.collect_all_errors = true;
        self
    }

    /// 获取诊断收集器
    pub fn diagnostics(&self) -> &DiagnosticCollector {
        &self.diagnostics
    }

    /// 创建详细的词法错误诊断
    fn create_lexer_diagnostic(&self, error_type: LexerErrorType, span: std::ops::Range<usize>) -> Diagnostic {
        let error_char = &self.source[span.clone()];
        let location = crate::diagnostic::SourceLocation::new(self.line, self.column);

        match error_type {
            LexerErrorType::InvalidCharacter => {
                Diagnostic::error(
                    ErrorCodes::LEXER_INVALID_CHARACTER,
                    CompilationPhase::Lexer,
                    format!("非法字符: '{}'", error_char),
                    location,
                )
                .with_details(format!(
                    "字符 '{}' 不是Cavvy语言支持的有效字符。Cavvy只支持标准ASCII字符集。",
                    error_char
                ))
                .with_suggestion(FixSuggestion::new("删除该字符或使用支持的字符替换"))
            }
            LexerErrorType::UnterminatedString => {
                Diagnostic::error(
                    ErrorCodes::LEXER_UNTERMINATED_STRING,
                    CompilationPhase::Lexer,
                    "未闭合的字符串字面量",
                    location,
                )
                .with_details("字符串字面量以双引号开始，但没有找到配对的结束双引号。")
                .with_suggestion(FixSuggestion::new("在字符串末尾添加双引号 (\")"))
            }
            LexerErrorType::InvalidEscapeSequence => {
                Diagnostic::error(
                    ErrorCodes::LEXER_INVALID_ESCAPE_SEQUENCE,
                    CompilationPhase::Lexer,
                    format!("无效的转义序列: '{}'", error_char),
                    location,
                )
                .with_details("Cavvy支持以下转义序列: \\n (换行), \\t (制表符), \\\" (双引号), \\\\ (反斜杠), \\' (单引号), \\0 (空字符)")
                .with_suggestion(FixSuggestion::new("使用有效的转义序列替换"))
            }
            LexerErrorType::InvalidNumberLiteral => {
                Diagnostic::error(
                    ErrorCodes::LEXER_INVALID_NUMBER_LITERAL,
                    CompilationPhase::Lexer,
                    format!("无效的数字字面量: '{}'", error_char),
                    location,
                )
                .with_details("数字字面量格式不正确。支持的格式: 十进制(123), 十六进制(0xFF), 二进制(0b101), 八进制(0o77)")
                .with_suggestion(FixSuggestion::new("检查数字格式，确保使用正确的进制前缀"))
            }
            LexerErrorType::UnterminatedComment => {
                Diagnostic::error(
                    ErrorCodes::LEXER_UNTERMINATED_COMMENT,
                    CompilationPhase::Lexer,
                    "未闭合的注释",
                    location,
                )
                .with_details("块注释以 /* 开始，但没有找到配对的结束标记 */。")
                .with_suggestion(FixSuggestion::new("添加 */ 结束注释，或将块注释改为行注释 //"))
            }
            LexerErrorType::InvalidIdentifier => {
                Diagnostic::error(
                    ErrorCodes::LEXER_INVALID_IDENTIFIER,
                    CompilationPhase::Lexer,
                    format!("无效的标识符: '{}'", error_char),
                    location,
                )
                .with_details("标识符必须以字母或下划线开头，后面可以跟字母、数字或下划线。")
                .with_suggestion(FixSuggestion::new("使用有效的标识符名称"))
            }
        }
    }

    pub fn tokenize(&mut self) -> cayResult<Vec<TokenWithLocation>> {
        let mut tokens = Vec::new();

        while let Some(token_result) = self.inner.next() {
            match token_result {
                Ok(token) => {
                    let span = self.inner.span();
                    let loc = SourceLocation {
                        line: self.line,
                        column: self.column,
                    };

                    // 更新行号和列号
                    if token == Token::Newline {
                        self.line += 1;
                        self.column = 1;
                        continue; // 不保留换行token
                    } else {
                        self.column += span.end - span.start;
                    }

                    // 检查源映射
                    let (source_file, source_line) = if let Some((file, line)) = self.source_map.get(&self.line) {
                        (Some(file.clone()), Some(*line))
                    } else {
                        (self.current_source_file.clone(), Some(self.line))
                    };

                    tokens.push(TokenWithLocation {
                        token,
                        loc,
                        source_file,
                        source_line,
                    });
                }
                Err(_) => {
                    let span = self.inner.span();
                    let error_char = &self.source[span.clone()];

                    // 检查源映射以获取正确的错误位置
                    let (error_line, error_file) = if let Some((file, line)) = self.source_map.get(&self.line) {
                        (*line, Some(file.clone()))
                    } else {
                        (self.line, self.current_source_file.clone())
                    };

                    if self.collect_all_errors {
                        // 收集错误但继续分析
                        let diagnostic = self.create_lexer_diagnostic(
                            LexerErrorType::InvalidCharacter,
                            span.clone()
                        );
                        self.diagnostics.add(diagnostic);

                        // 跳过这个字符继续
                        self.column += span.end - span.start;
                    } else {
                        // 立即返回错误（保持向后兼容）
                        let error_msg = if let Some(ref file) = error_file {
                            format!("Unexpected character: '{}' in {}:{}", error_char, file, error_line)
                        } else {
                            format!("Unexpected character: '{}' at line {}", error_char, error_line)
                        };
                        return Err(lexer_error(
                            error_line,
                            self.column,
                            error_msg
                        ));
                    }
                }
            }
        }

        // 检查是否有收集到的错误
        if self.diagnostics.has_errors() {
            return Err(lexer_error(
                self.line,
                self.column,
                format!("词法分析发现 {} 个错误", self.diagnostics.error_count())
            ));
        }

        // 添加EOF标记 - 使用Identifier作为哨兵值
        let (source_file, source_line) = if let Some((file, line)) = self.source_map.get(&self.line) {
            (Some(file.clone()), Some(*line))
        } else {
            (self.current_source_file.clone(), Some(self.line))
        };

        tokens.push(TokenWithLocation {
            token: Token::Identifier(String::new()), // 用作EOF标记
            loc: SourceLocation {
                line: self.line,
                column: self.column,
            },
            source_file,
            source_line,
        });

        Ok(tokens)
    }

    /// 检查未闭合的字符串字面量
    pub fn check_unterminated_strings(&mut self) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut in_string = false;
        let mut string_start_line = 0;
        let mut string_start_col = 0;
        let mut chars = self.source.chars().peekable();
        let mut line = 1;
        let mut col = 1;

        while let Some(c) = chars.next() {
            match c {
                '"' if !in_string => {
                    in_string = true;
                    string_start_line = line;
                    string_start_col = col;
                }
                '"' if in_string => {
                    // 检查是否是转义的引号
                    let mut backslash_count = 0;
                    let mut check_col = col - 1;
                    for ch in self.source.lines().nth(line - 1).unwrap_or("").chars().rev() {
                        if check_col == 0 { break; }
                        if ch == '\\' {
                            backslash_count += 1;
                            check_col -= 1;
                        } else {
                            break;
                        }
                    }
                    if backslash_count % 2 == 0 {
                        in_string = false;
                    }
                }
                '\n' if in_string => {
                    // 字符串跨行是错误
                    diagnostics.push(
                        Diagnostic::error(
                            ErrorCodes::LEXER_UNTERMINATED_STRING,
                            CompilationPhase::Lexer,
                            "字符串字面量不能跨行",
                            crate::diagnostic::SourceLocation::new(line, col),
                        )
                        .with_related_info(
                            "字符串开始位置",
                            crate::diagnostic::SourceLocation::new(string_start_line, string_start_col)
                        )
                    );
                    in_string = false;
                }
                '\n' => {
                    line += 1;
                    col = 0;
                }
                _ => {}
            }
            col += 1;
        }

        // 检查文件结束时是否还在字符串中
        if in_string {
            diagnostics.push(
                Diagnostic::error(
                    ErrorCodes::LEXER_UNTERMINATED_STRING,
                    CompilationPhase::Lexer,
                    "未闭合的字符串字面量（到达文件末尾）",
                    crate::diagnostic::SourceLocation::new(string_start_line, string_start_col),
                )
            );
        }

        diagnostics
    }
}

/// 从预处理器结果中提取源映射
/// 预处理器生成的格式是每行代码对应一个源位置
fn extract_source_map_from_preprocessed(source: &str) -> std::collections::HashMap<usize, (String, usize)> {
    let mut source_map = std::collections::HashMap::new();
    let lines: Vec<&str> = source.lines().collect();

    for (output_line, line_content) in lines.iter().enumerate() {
        let output_line_num = output_line + 1; // 1-based

        // 查找源映射信息
        // 格式：我们需要根据预处理器的SourceMap来重建映射
        // 由于预处理器现在直接将源映射信息嵌入到行中，我们需要解析它
        // 但更简单的方法是使用预处理器返回的SourceMap
        // 这里我们暂时不解析，而是在lib.rs中直接使用预处理器返回的SourceMap
    }

    source_map
}

pub fn lex(source: &str) -> cayResult<Vec<TokenWithLocation>> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize()
}

/// 使用源映射进行词法分析
pub fn lex_with_source_map(source: &str, source_map: std::collections::HashMap<usize, (String, usize)>) -> cayResult<Vec<TokenWithLocation>> {
    let mut lexer = Lexer::with_source_map(source, source_map);
    lexer.tokenize()
}

/// 使用诊断收集的词法分析
pub fn lex_with_diagnostics(source: &str) -> (cayResult<Vec<TokenWithLocation>>, DiagnosticCollector) {
    let mut lexer = Lexer::new(source).with_collect_all_errors();

    // 首先检查未闭合的字符串
    let string_diagnostics = lexer.check_unterminated_strings();
    for diag in string_diagnostics {
        lexer.diagnostics.add(diag);
    }

    let result = lexer.tokenize();
    (result, lexer.diagnostics)
}

/// 处理字符串中的转义序列
fn process_escape_sequences(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('\'') => result.push('\''),
                Some('0') => result.push('\0'),
                Some(other) => {
                    // 对于不认识的转义序列，保留原样
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    
    result
}

/// 处理字符字面量的转义序列
fn process_char_escape(s: &str) -> Option<char> {
    if s.starts_with("\\") {
        match s.chars().nth(1) {
            Some('n') => Some('\n'),
            Some('t') => Some('\t'),
            Some('r') => Some('\r'),
            Some('\\') => Some('\\'),
            Some('\'') => Some('\''),
            Some('"') => Some('"'),
            Some('0') => Some('\0'),
            _ => s.chars().nth(1),
        }
    } else {
        s.chars().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic() {
        let source = r#"int x = 42;"#;
        let tokens = lex(source).unwrap();
        assert!(tokens.len() >= 5);
    }

    #[test]
    fn test_lexer_invalid_character() {
        let source = "int x = 42 @;";
        let result = lex(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_lexer_unterminated_string() {
        let source = r#"String s = "hello;"#;
        let (_, diagnostics) = lex_with_diagnostics(source);
        // 应该检测到未闭合的字符串
        assert!(diagnostics.diagnostics().iter().any(|d| d.code == ErrorCodes::LEXER_UNTERMINATED_STRING));
    }
}
