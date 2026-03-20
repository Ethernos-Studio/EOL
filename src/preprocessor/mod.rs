//! Cavvy 预处理器模块
//!
//! 实现 0.3.5.0 版本的预处理指令系统：
//! - #include "path"  - 文件包含（隐式 #pragma once）
//! - #define NAME value  - 常量定义（无参数宏）
//! - #ifdef / #ifndef / #else / #elif / #endif  - 条件编译
//! - #error "message"  - 编译期错误
//! - #warning "message"  - 编译期警告
//!
//! 设计约束：
//! - 仅支持简单常量定义，禁止宏函数
//! - 隐式 #pragma once 基于绝对路径哈希
//! - 预处理在词法分析之前执行，生成纯源代码
//! - 生成 #source <file> <line> 标记以支持源映射

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use crate::error::{cayResult, cayError};

/// 源位置信息
#[derive(Debug, Clone)]
pub struct SourcePosition {
    pub file: String,
    pub line: usize,
}

/// 源映射表：将输出行号映射到原始源位置
#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    pub mappings: Vec<SourcePosition>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }

    /// 添加一个源位置映射
    pub fn add_mapping(&mut self, file: String, line: usize) {
        self.mappings.push(SourcePosition { file, line });
    }

    /// 获取指定输出行号对应的源位置
    pub fn get_source_position(&self, output_line: usize) -> Option<&SourcePosition> {
        // output_line 是1-based的
        self.mappings.get(output_line.saturating_sub(1))
    }

    /// 获取映射数量
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    /// 序列化为字符串（用于嵌入到预处理后的代码中）
    pub fn serialize(&self) -> String {
        self.mappings.iter()
            .map(|pos| format!("#source {} {}", pos.file, pos.line))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// 预处理结果，包含处理后的代码和源映射
#[derive(Debug, Clone)]
pub struct PreprocessResult {
    pub code: String,
    pub source_map: SourceMap,
}

/// 预处理器状态
pub struct Preprocessor {
    /// 已定义的宏常量 (name -> value)
    defines: HashMap<String, String>,
    /// 已包含的文件路径集合（用于 #pragma once 语义）
    included_files: HashSet<String>,
    /// 基础目录（用于解析相对路径）
    base_dir: PathBuf,
    /// 当前条件编译栈
    conditional_stack: Vec<ConditionalState>,
    /// 是否处于被跳过的代码块中
    skipping: bool,
    /// 包含栈（用于循环包含检测和错误报告）
    include_stack: Vec<String>,
    /// 系统包含路径列表
    system_include_paths: Vec<PathBuf>,
}

/// 条件编译状态
#[derive(Debug, Clone, Copy, PartialEq)]
enum ConditionalState {
    /// 当前条件为真，正在处理代码
    Active,
    /// 当前条件为假，但处于可能执行 #else 的链中
    Inactive,
    /// 当前条件为假，且已经执行过某个分支，跳过后续所有 #elif/#else
    Done,
}

/// 预处理指令类型
#[derive(Debug, Clone)]
enum Directive {
    /// #include "path" 或 #include <path>
    /// bool 表示是否是系统包含路径 (<>)
    Include(String, bool),
    /// #define name value
    Define(String, String),
    /// #ifdef name
    Ifdef(String),
    /// #ifndef name
    Ifndef(String),
    /// #if expression
    If(String),
    /// #else
    Else,
    /// #elif expression
    Elif(String),
    /// #endif
    Endif,
    /// #error "message"
    Error(String),
    /// #warning "message"
    Warning(String),
}

impl Preprocessor {
    /// 创建新的预处理器实例
    /// 
    /// # Arguments
    /// * `base_dir` - 源代码基础目录，用于解析相对路径
    /// 
    /// # Returns
    /// 初始化后的预处理器
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            defines: HashMap::new(),
            included_files: HashSet::new(),
            base_dir: base_dir.as_ref().to_path_buf(),
            conditional_stack: Vec::new(),
            skipping: false,
            include_stack: Vec::new(),
            system_include_paths: Vec::new(),
        }
    }

    /// 创建带有系统包含路径的预处理器实例
    /// 
    /// # Arguments
    /// * `base_dir` - 源代码基础目录
    /// * `system_paths` - 系统包含路径列表
    /// 
    /// # Returns
    /// 初始化后的预处理器
    pub fn with_system_paths(base_dir: impl AsRef<Path>, system_paths: Vec<PathBuf>) -> Self {
        Self {
            defines: HashMap::new(),
            included_files: HashSet::new(),
            base_dir: base_dir.as_ref().to_path_buf(),
            conditional_stack: Vec::new(),
            skipping: false,
            include_stack: Vec::new(),
            system_include_paths: system_paths,
        }
    }

    /// 预处理源文件，返回处理后的源代码（带源映射）
    ///
    /// # Arguments
    /// * `source` - 原始源代码
    /// * `file_path` - 源文件路径（用于错误报告）
    ///
    /// # Returns
    /// 预处理后的结果，包含代码和源映射
    ///
    /// # Errors
    /// 当遇到无效指令或文件无法读取时返回错误
    pub fn process_with_source_map(&mut self, source: &str, file_path: &str) -> cayResult<PreprocessResult> {
        // 将当前文件压入包含栈
        self.include_stack.push(file_path.to_string());

        let result = self.process_internal_with_source_map(source, file_path);

        // 弹出当前文件
        self.include_stack.pop();

        result
    }

    /// 预处理源文件，返回处理后的源代码（向后兼容）
    pub fn process(&mut self, source: &str, file_path: &str) -> cayResult<String> {
        let result = self.process_with_source_map(source, file_path)?;
        Ok(result.code)
    }

    /// 内部处理函数（带源映射）
    fn process_internal_with_source_map(&mut self, source: &str, file_path: &str) -> cayResult<PreprocessResult> {
        let lines: Vec<&str> = source.lines().collect();
        let mut output_lines = Vec::new();
        let mut source_map = SourceMap::new();

        for (line_num, line) in lines.iter().enumerate() {
            let line_number = line_num + 1;

            // 检查是否是预处理指令行
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                match self.parse_directive(trimmed, line_number, file_path) {
                    Ok(Some(directive)) => {
                        self.process_directive_with_source_map(directive, &mut output_lines, &mut source_map, file_path)?;
                    }
                    Ok(None) => {
                        // 跳过空指令（如纯注释），但仍添加源映射
                        source_map.add_mapping(file_path.to_string(), line_number);
                        output_lines.push("".to_string());
                    }
                    Err(e) => return Err(e),
                }
            } else if self.skipping {
                // 处于条件编译跳过状态，不输出代码行
                // 但仍需跟踪行号以保持行号映射
                source_map.add_mapping(file_path.to_string(), line_number);
                output_lines.push("".to_string());
            } else {
                // 普通代码行，进行宏替换后输出
                let processed = self.expand_macros(line);
                source_map.add_mapping(file_path.to_string(), line_number);
                output_lines.push(processed);
            }
        }

        // 检查条件编译栈是否为空
        if !self.conditional_stack.is_empty() {
            return Err(cayError::Preprocessor {
                line: lines.len(),
                column: 1,
                message: "未闭合的条件编译指令，缺少 #endif".to_string(),
                suggestion: "请为每个 #ifdef 或 #ifndef 添加对应的 #endif".to_string(),
            });
        }

        Ok(PreprocessResult {
            code: output_lines.join("\n"),
            source_map,
        })
    }

    /// 解析单行预处理指令
    /// 
    /// # Arguments
    /// * `line` - 已去除前导空白的行内容
    /// * `line_num` - 行号（用于错误报告）
    /// * `file_path` - 文件路径（用于错误报告）
    /// 
    /// # Returns
    /// 解析出的指令或 None
    fn parse_directive(&self, line: &str, line_num: usize, _file_path: &str) -> cayResult<Option<Directive>> {
        // 去除 # 后面的空白
        let content = line[1..].trim_start();
        
        if content.is_empty() {
            return Ok(None);
        }
        
        // 提取指令名和参数
        let mut parts = content.splitn(2, |c: char| c.is_whitespace());
        let directive_name = parts.next().unwrap_or("");
        let args = parts.next().unwrap_or("").trim();
        
        match directive_name {
            "include" => {
                // 解析 #include "path" 或 #include <path>
                let (path, is_system) = self.parse_include_path(args, line_num)?;
                Ok(Some(Directive::Include(path, is_system)))
            }
            "define" => {
                // 解析 #define name value
                let (name, value) = self.parse_define_args(args, line_num)?;
                Ok(Some(Directive::Define(name, value)))
            }
            "ifdef" => {
                let name = self.parse_identifier(args, line_num)?;
                Ok(Some(Directive::Ifdef(name)))
            }
            "ifndef" => {
                let name = self.parse_identifier(args, line_num)?;
                Ok(Some(Directive::Ifndef(name)))
            }
            "if" => {
                let expr = args.trim().to_string();
                Ok(Some(Directive::If(expr)))
            }
            "else" => {
                if !args.is_empty() {
                    return Err(cayError::Preprocessor {
                        line: line_num,
                        column: 1,
                        message: "#else 指令不接受参数".to_string(),
                        suggestion: "使用 #else 而不是 #else CONDITION".to_string(),
                    });
                }
                Ok(Some(Directive::Else))
            }
            "elif" => {
                let name = self.parse_identifier(args, line_num)?;
                Ok(Some(Directive::Elif(name)))
            }
            "endif" => {
                if !args.is_empty() {
                    return Err(cayError::Preprocessor {
                        line: line_num,
                        column: 1,
                        message: "#endif 指令不接受参数".to_string(),
                        suggestion: "使用 #endif 而不是 #endif CONDITION".to_string(),
                    });
                }
                Ok(Some(Directive::Endif))
            }
            "error" => {
                let message = self.parse_string_literal(args, line_num)?;
                Ok(Some(Directive::Error(message)))
            }
            "warning" => {
                let message = self.parse_string_literal(args, line_num)?;
                Ok(Some(Directive::Warning(message)))
            }
            _ => {
                Err(cayError::Preprocessor {
                    line: line_num,
                    column: 1,
                    message: format!("未知的预处理指令: {}", directive_name),
                    suggestion: "支持的指令: #include, #define, #ifdef, #ifndef, #else, #elif, #endif, #error, #warning".to_string(),
                })
            }
        }
    }

    /// 解析字符串字面量（用于 #error, #warning）
    fn parse_string_literal(&self, args: &str, line_num: usize) -> cayResult<String> {
        let trimmed = args.trim();
        if trimmed.len() < 2 {
            return Err(cayError::Preprocessor {
                line: line_num,
                column: 1,
                message: "缺少字符串参数".to_string(),
                suggestion: "使用双引号包围字符串，例如: \"path/to/file.cay\"".to_string(),
            });
        }
        
        // 只支持双引号
        if !trimmed.starts_with('"') || !trimmed.ends_with('"') {
            return Err(cayError::Preprocessor {
                line: line_num,
                column: 1,
                message: "参数必须是双引号字符串".to_string(),
                suggestion: "使用 \"path\" 而不是 <path>".to_string(),
            });
        }
        
        Ok(trimmed[1..trimmed.len()-1].to_string())
    }

    /// 解析 #include 路径（支持 "path" 和 <path>）
    fn parse_include_path(&self, args: &str, line_num: usize) -> cayResult<(String, bool)> {
        let trimmed = args.trim();
        if trimmed.len() < 2 {
            return Err(cayError::Preprocessor {
                line: line_num,
                column: 1,
                message: "缺少文件路径参数".to_string(),
                suggestion: "使用 #include \"path\" 或 #include <path>".to_string(),
            });
        }
        
        // 检查是双引号还是尖括号
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            // 双引号包含：本地文件
            Ok((trimmed[1..trimmed.len()-1].to_string(), false))
        } else if trimmed.starts_with('<') && trimmed.ends_with('>') {
            // 尖括号包含：系统文件
            Ok((trimmed[1..trimmed.len()-1].to_string(), true))
        } else {
            Err(cayError::Preprocessor {
                line: line_num,
                column: 1,
                message: "#include 参数格式错误".to_string(),
                suggestion: "使用 #include \"path\" 或 #include <path>".to_string(),
            })
        }
    }

    /// 解析标识符
    fn parse_identifier(&self, args: &str, line_num: usize) -> cayResult<String> {
        let trimmed = args.trim();
        if trimmed.is_empty() {
            return Err(cayError::Preprocessor {
                line: line_num,
                column: 1,
                message: "缺少标识符参数".to_string(),
                suggestion: "提供标识符名称，例如: #ifdef DEBUG".to_string(),
            });
        }
        
        // 检查是否是有效的标识符
        let first_char = trimmed.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return Err(cayError::Preprocessor {
                line: line_num,
                column: 1,
                message: format!("无效的标识符: {}", trimmed),
                suggestion: "标识符必须以字母或下划线开头".to_string(),
            });
        }
        
        // 只取第一个标识符（后面的内容忽略）
        let ident: String = trimmed.chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        
        Ok(ident)
    }

    /// 解析 #define 的参数
    fn parse_define_args(&self, args: &str, line_num: usize) -> cayResult<(String, String)> {
        let trimmed = args.trim();
        if trimmed.is_empty() {
            return Err(cayError::Preprocessor {
                line: line_num,
                column: 1,
                message: "#define 缺少宏名称".to_string(),
                suggestion: "使用格式: #define NAME value 或 #define NAME".to_string(),
            });
        }
        
        // 查找第一个空白字符分隔名称和值
        let mut parts = trimmed.splitn(2, |c: char| c.is_whitespace());
        let name = parts.next().unwrap_or("").to_string();
        let value = parts.next().unwrap_or("").trim().to_string();
        
        // 检查名称是否包含括号（禁止宏函数）
        if name.contains('(') {
            return Err(cayError::Preprocessor {
                line: line_num,
                column: 1,
                message: "不支持宏函数".to_string(),
                suggestion: "Cavvy 预处理器仅支持简单常量定义，使用 static final 方法代替".to_string(),
            });
        }
        
        // 验证标识符格式
        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return Err(cayError::Preprocessor {
                line: line_num,
                column: 1,
                message: format!("无效的宏名称: {}", name),
                suggestion: "宏名称必须以字母或下划线开头".to_string(),
            });
        }
        
        Ok((name, value))
    }

    /// 处理预处理指令（带源映射）
    fn process_directive_with_source_map(
        &mut self,
        directive: Directive,
        output_lines: &mut Vec<String>,
        source_map: &mut SourceMap,
        file_path: &str,
    ) -> cayResult<()> {
        match directive {
            Directive::Include(path, is_system) => {
                if !self.skipping {
                    self.handle_include_with_source_map(&path, is_system, output_lines, source_map, file_path)?;
                }
            }
            Directive::Define(name, value) => {
                if !self.skipping {
                    self.defines.insert(name, value);
                }
            }
            Directive::Ifdef(name) => {
                let should_process = !self.skipping && self.defines.contains_key(&name);
                self.push_conditional(should_process);
            }
            Directive::Ifndef(name) => {
                let should_process = !self.skipping && !self.defines.contains_key(&name);
                self.push_conditional(should_process);
            }
            Directive::If(expr) => {
                let result = self.evaluate_if_expression(&expr);
                self.push_conditional(!self.skipping && result);
            }
            Directive::Else => {
                self.handle_else()?;
            }
            Directive::Elif(name) => {
                self.handle_elif(name)?;
            }
            Directive::Endif => {
                self.pop_conditional()?;
            }
            Directive::Error(message) => {
                if !self.skipping {
                    return Err(cayError::Preprocessor {
                        line: 0,
                        column: 0,
                        message: format!("#error: {}", message),
                        suggestion: "根据编译条件移除此错误或修改预处理器条件".to_string(),
                    });
                }
            }
            Directive::Warning(message) => {
                if !self.skipping {
                    // 使用标准警告输出函数
                    crate::error::print_warning(&format!("预处理器警告: {}", message));
                }
            }
        }
        Ok(())
    }

    /// 处理 #else 指令
    fn handle_else(&mut self) -> cayResult<()> {
        if self.conditional_stack.is_empty() {
            return Err(cayError::Preprocessor {
                line: 0,
                column: 0,
                message: "#else 没有对应的 #ifdef 或 #ifndef".to_string(),
                suggestion: "确保 #else 在 #ifdef 或 #ifndef 块内".to_string(),
            });
        }

        let last_idx = self.conditional_stack.len() - 1;
        let current_state = self.conditional_stack[last_idx];

        match current_state {
            ConditionalState::Active => {
                // 当前分支已执行，跳过后续
                self.conditional_stack[last_idx] = ConditionalState::Done;
            }
            ConditionalState::Inactive => {
                // 当前分支未执行，开始执行 else 分支
                self.conditional_stack[last_idx] = ConditionalState::Active;
            }
            ConditionalState::Done => {
                // 已经执行过某个分支，保持跳过状态
            }
        }

        self.update_skipping_state();
        Ok(())
    }

    /// 处理 #elif 指令
    fn handle_elif(&mut self, name: String) -> cayResult<()> {
        if self.conditional_stack.is_empty() {
            return Err(cayError::Preprocessor {
                line: 0,
                column: 0,
                message: "#elif 没有对应的 #ifdef 或 #ifndef".to_string(),
                suggestion: "确保 #elif 在 #ifdef 或 #ifndef 块内".to_string(),
            });
        }

        let last_idx = self.conditional_stack.len() - 1;
        let current_state = self.conditional_stack[last_idx];

        match current_state {
            ConditionalState::Active => {
                // 当前分支已执行，跳过后续所有分支
                self.conditional_stack[last_idx] = ConditionalState::Done;
            }
            ConditionalState::Inactive => {
                // 检查条件是否满足
                if self.defines.contains_key(&name) {
                    self.conditional_stack[last_idx] = ConditionalState::Active;
                }
                // 否则保持 Inactive，等待下一个 #elif 或 #else
            }
            ConditionalState::Done => {
                // 已经执行过某个分支，保持跳过状态
            }
        }

        self.update_skipping_state();
        Ok(())
    }

    /// 评估 #if 表达式
    fn evaluate_if_expression(&self, expr: &str) -> bool {
        let trimmed = expr.trim();
        
        // 简单表达式求值
        // 支持: defined(MACRO), MACRO == value, MACRO != value
        
        // 检查 defined(MACRO)
        if trimmed.starts_with("defined(") && trimmed.ends_with(")") {
            let macro_name = &trimmed[8..trimmed.len()-1];
            return self.defines.contains_key(macro_name);
        }
        
        // 检查数字字面量（非0为真）
        if let Ok(num) = trimmed.parse::<i64>() {
            return num != 0;
        }
        
        // 检查宏是否存在且非空/非0
        if let Some(value) = self.defines.get(trimmed) {
            if value.is_empty() {
                return true; // 空定义视为真
            }
            if let Ok(num) = value.parse::<i64>() {
                return num != 0;
            }
            return true; // 非空字符串视为真
        }
        
        // 支持简单比较: MACRO == value
        if trimmed.contains("==") {
            let parts: Vec<&str> = trimmed.split("==").collect();
            if parts.len() == 2 {
                let left = parts[0].trim();
                let right = parts[1].trim();
                let left_val = self.defines.get(left).map(|s| s.as_str()).unwrap_or(left);
                return left_val == right;
            }
        }
        
        // 默认：未定义的宏视为假
        false
    }

    /// 更新跳过状态
    fn update_skipping_state(&mut self) {
        self.skipping = self.conditional_stack.iter()
            .any(|state| *state != ConditionalState::Active);
    }

    /// 处理 #include 指令（带源映射）
    fn handle_include_with_source_map(
        &mut self,
        path: &str,
        is_system: bool,
        output_lines: &mut Vec<String>,
        source_map: &mut SourceMap,
        current_file: &str,
    ) -> cayResult<()> {
        // 解析完整路径
        let include_path = self.resolve_include_path(path, is_system, current_file)?;

        // 标准化路径用于去重检查
        let canonical_path = include_path.canonicalize()
            .map_err(|e| cayError::Io(
                format!("无法解析包含路径 '{}': {}", path, e)
            ))?;

        let path_key = canonical_path.to_string_lossy().to_string();

        // 检查循环包含
        if self.include_stack.contains(&path_key) {
            let chain = self.include_stack.join(" -> ");
            return Err(cayError::Preprocessor {
                line: 0,
                column: 0,
                message: format!("检测到循环包含: {}", path_key),
                suggestion: format!("包含链: {} -> {}", chain, path_key),
            });
        }

        // 隐式 #pragma once: 检查是否已包含
        if self.included_files.contains(&path_key) {
            return Ok(());
        }

        // 读取文件内容
        let content = std::fs::read_to_string(&canonical_path)
            .map_err(|e| cayError::Io(
                format!("无法读取包含文件 '{}': {}", path, e)
            ))?;

        // 标记为已包含
        self.included_files.insert(path_key.clone());

        // 递归处理被包含的文件（带源映射）
        let sub_path = canonical_path.to_string_lossy();
        let result = self.process_with_source_map(&content, &sub_path)?;

        // 添加被包含文件的源映射和代码
        // 为每一行添加对应的源映射
        for mapping in &result.source_map.mappings {
            source_map.add_mapping(mapping.file.clone(), mapping.line);
        }

        // 将被包含文件的代码按行分割并逐行添加
        // 这样可以确保源映射和代码行一一对应
        let included_lines: Vec<&str> = result.code.lines().collect();
        for line in included_lines {
            output_lines.push(line.to_string());
        }

        Ok(())
    }

    /// 解析包含路径
    /// 
    /// 对于 #include "path" (is_system = false):
    /// 1. 如果是绝对路径，直接使用
    /// 2. 相对于当前文件目录
    /// 3. 相对于基础目录
    /// 4. 系统包含路径
    ///
    /// 对于 #include <path> (is_system = true):
    /// 1. 如果是绝对路径，直接使用
    /// 2. 系统包含路径（优先）
    /// 3. 相对于基础目录
    fn resolve_include_path(&self, path: &str, is_system: bool, current_file: &str) -> cayResult<PathBuf> {
        // 1. 绝对路径
        if Path::new(path).is_absolute() {
            return Ok(PathBuf::from(path));
        }
        
        if is_system {
            // #include <path> 的搜索顺序
            
            // 2. 系统包含路径（优先）
            for sys_path in &self.system_include_paths {
                let sys_include_path = sys_path.join(path);
                if sys_include_path.exists() {
                    return Ok(sys_include_path);
                }
            }
            
            // 3. 相对于基础目录
            let base_path = self.base_dir.join(path);
            if base_path.exists() {
                return Ok(base_path);
            }
        } else {
            // #include "path" 的搜索顺序
            
            // 2. 相对于当前文件目录
            if let Some(current_dir) = Path::new(current_file).parent() {
                let relative_path = current_dir.join(path);
                if relative_path.exists() {
                    return Ok(relative_path);
                }
            }
            
            // 3. 相对于基础目录
            let base_path = self.base_dir.join(path);
            if base_path.exists() {
                return Ok(base_path);
            }
            
            // 4. 系统包含路径
            for sys_path in &self.system_include_paths {
                let sys_include_path = sys_path.join(path);
                if sys_include_path.exists() {
                    return Ok(sys_include_path);
                }
            }
        }
        
        // 如果都找不到，返回相对于当前文件的路径（让后续错误处理报告）
        let current_dir = Path::new(current_file).parent()
            .unwrap_or(&self.base_dir);
        Ok(current_dir.join(path))
    }

    /// 获取当前包含栈（用于错误报告）
    pub fn get_include_stack(&self) -> &[String] {
        &self.include_stack
    }

    /// 压入条件编译状态
    fn push_conditional(&mut self, should_process: bool) {
        let state = if should_process {
            ConditionalState::Active
        } else {
            ConditionalState::Inactive
        };
        self.conditional_stack.push(state);
        self.update_skipping_state();
    }

    /// 弹出条件编译状态
    fn pop_conditional(&mut self) -> cayResult<()> {
        if self.conditional_stack.pop().is_none() {
            return Err(cayError::Preprocessor {
                line: 0,
                column: 0,
                message: "多余的 #endif".to_string(),
                suggestion: "确保每个 #endif 都有对应的 #ifdef 或 #ifndef".to_string(),
            });
        }
        
        self.update_skipping_state();
        Ok(())
    }

    /// 展开宏定义（简单的文本替换）
    fn expand_macros(&self, line: &str) -> String {
        let mut result = line.to_string();
        
        // 按名称长度降序排序，避免短名称替换干扰长名称
        let mut macros: Vec<(&String, &String)> = self.defines.iter().collect();
        macros.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        
        for (name, value) in macros {
            // 简单的字符串替换
            // 注意：这不处理注释、字符串字面量等边界情况
            // 对于 0.3.5.0 版本，这是可接受的简化
            result = result.replace(name, value);
        }
        
        result
    }
}

/// 便捷的预处理函数（带源映射）
///
/// 创建一个临时预处理器实例并处理源代码
///
/// # Arguments
/// * `source` - 原始源代码
/// * `file_path` - 源文件路径（用于错误报告）
/// * `base_dir` - 源代码基础目录，用于解析相对路径
///
/// # Returns
/// 预处理后的结果，包含代码和源映射
pub fn preprocess_with_source_map(source: &str, file_path: &str, base_dir: impl AsRef<Path>) -> cayResult<PreprocessResult> {
    let mut preprocessor = Preprocessor::new(base_dir);
    preprocessor.process_with_source_map(source, file_path)
}

/// 便捷的预处理函数（向后兼容）
///
/// 创建一个临时预处理器实例并处理源代码
///
/// # Arguments
/// * `source` - 原始源代码
/// * `file_path` - 源文件路径（用于错误报告）
/// * `base_dir` - 源代码基础目录，用于解析相对路径
///
/// # Returns
/// 预处理后的源代码字符串
pub fn preprocess(source: &str, file_path: &str, base_dir: impl AsRef<Path>) -> cayResult<String> {
    let mut preprocessor = Preprocessor::new(base_dir);
    preprocessor.process(source, file_path)
}
