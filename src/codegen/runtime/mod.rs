//! 运行时支持函数生成模块
//!
//! 本模块包含所有 cay 运行时支持函数的 LLVM IR 生成。
//! 每个运行时函数都有独立的子模块。

use crate::codegen::context::IRGenerator;

// 子模块声明
mod string_concat;
mod float_to_string;
mod int_to_string;
mod bool_to_string;
mod char_to_string;
mod string_length;
mod string_substring;
mod string_indexof;
mod string_charat;
mod string_replace;

impl IRGenerator {
    /// 发射IR头部（外部声明和运行时函数）
    pub fn emit_header(&mut self) {
        self.emit_raw("; cay (Ethernos Object Language) Generated LLVM IR");
        
        // 根据目标平台设置目标三元组
        let target_triple = if let Some(config) = &self.platform_config {
            match config.target_os.as_str() {
                "windows" => "x86_64-w64-mingw32",
                "linux" => "x86_64-unknown-linux-gnu",
                "macos" => "x86_64-apple-darwin",
                _ => "x86_64-unknown-linux-gnu"
            }
        } else if cfg!(target_os = "windows") {
            "x86_64-w64-mingw32"
        } else if cfg!(target_os = "linux") {
            "x86_64-unknown-linux-gnu"
        } else if cfg!(target_os = "macos") {
            "x86_64-apple-darwin"
        } else {
            "x86_64-unknown-linux-gnu"
        };
        self.emit_raw(&format!("target triple = \"{}\"", target_triple));
        self.emit_raw("");

        // 声明外部函数 (printf 和标准C库函数)
        self.emit_raw("declare i32 @printf(i8*, ...)");
        self.emit_raw("declare i32 @scanf(i8*, ...)");
        
        // 根据平台配置声明平台特定函数
        let platform_declarations = if let Some(config) = &self.platform_config {
            let mut declarations = String::new();
            match config.target_os.as_str() {
                "windows" => {
                    // Windows 平台总是声明 SetConsoleOutputCP，因为 generator.rs 中总是调用它
                    declarations.push_str("declare dllimport void @SetConsoleOutputCP(i32)\n");
                    if config.is_defined("WINDOWS_SPECIFIC") {
                        declarations.push_str("declare void @WindowsSpecificInit()\n");
                    }
                }
                "linux" | "macos" => {
                    if config.is_feature_enabled("console_utf8") {
                        declarations.push_str("declare i8* @setlocale(i32, i8*)\n");
                        declarations.push_str("@.str.locale = private unnamed_addr constant [6 x i8] c\"C.UTF-8\"\00\n");
                    }
                    if config.is_defined("LINUX_SPECIFIC") {
                        declarations.push_str("declare void @LinuxSpecificInit()\n");
                    }
                    if config.is_defined("MACOS_SPECIFIC") {
                        declarations.push_str("declare void @MacOSSpecificInit()\n");
                    }
                }
                _ => {}
            }
            declarations
        } else if self.target_triple.contains("windows") || self.target_triple.contains("mingw32") {
            // 向后兼容：如果没有平台配置，使用目标三元组判断
            "declare void @SetConsoleOutputCP(i32)\n".to_string()
        } else {
            "".to_string()
        };
        
        // 发射宏定义
        if let Some(config) = &self.platform_config {
            let mut has_macros = false;
            let defines = config.defines.clone(); // 克隆以避免借用冲突
            let undefines = config.undefines.clone(); // 克隆以避免借用冲突
            
            for define in &defines {
                if !undefines.contains(define) {
                    self.emit_raw(&format!("; #define {}", define));
                    has_macros = true;
                }
            }
            if has_macros {
                self.emit_raw("");
            }
        }

        // 发射平台特定声明
        if !platform_declarations.is_empty() {
            self.emit_raw(&platform_declarations);
        }
        
        self.emit_raw("declare i64 @strlen(i8*)");
        self.emit_raw("declare i8* @calloc(i64, i64)");
        self.emit_raw("declare void @exit(i32)");
        self.emit_raw("declare void @llvm.memcpy.p0i8.p0i8.i64(i8* noalias nocapture writeonly, i8* noalias nocapture readonly, i64, i1 immarg)");
        self.emit_raw("declare i32 @snprintf(i8*, i64, i8*, ...)");
        self.emit_raw("@.str.float_fmt = private unnamed_addr constant [3 x i8] c\"%f\\00\", align 1");
        self.emit_raw("@.str.int_fmt = private unnamed_addr constant [5 x i8] c\"%lld\\00\", align 1");
        self.emit_raw("@.str.true_str = private unnamed_addr constant [5 x i8] c\"true\\00\", align 1");
        self.emit_raw("@.str.false_str = private unnamed_addr constant [6 x i8] c\"false\\00\", align 1");
        self.emit_raw("");

        // 空字符串常量（用于 null 安全）
        self.emit_raw("@.cay_empty_str = private unnamed_addr constant [1 x i8] c\"\\00\", align 1");
        self.emit_raw("");

        // 生成运行时函数
        self.emit_string_concat_runtime();
        self.emit_float_to_string_runtime();
        self.emit_int_to_string_runtime();
        self.emit_bool_to_string_runtime();
        self.emit_char_to_string_runtime();
        self.emit_string_length_runtime();
        self.emit_string_substring_runtime();
        self.emit_string_indexof_runtime();
        self.emit_string_charat_runtime();
        self.emit_string_replace_runtime();
    }
}