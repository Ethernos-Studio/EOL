pub mod error;
pub mod types;
pub mod ast;
pub mod preprocessor;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod codegen;

use std::path::{Path, PathBuf};
use error::cayResult;

/// 编译器配置选项
#[derive(Debug, Clone)]
pub struct CompilerOptions {
    pub target_os: String,
    pub features: Vec<String>,
    pub no_features: Vec<String>,
    pub defines: Vec<String>,
    pub undefines: Vec<String>,
    pub obfuscate: bool,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            target_os: std::env::consts::OS.to_string(),
            features: Vec::new(),
            no_features: Vec::new(),
            defines: Vec::new(),
            undefines: Vec::new(),
            obfuscate: false,
        }
    }
}

pub struct Compiler {
    options: CompilerOptions,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            options: CompilerOptions::default(),
        }
    }
    
    pub fn with_options(options: CompilerOptions) -> Self {
        Self { options }
    }

    /// 编译源代码为 LLVM IR
    /// 
    /// # Arguments
    /// * `source` - 原始源代码（已预处理）
    /// * `output_path` - 输出文件路径
    /// 
    /// # Returns
    /// 编译成功返回 Ok(())
    pub fn compile(&self, source: &str, output_path: &str) -> cayResult<()> {
        // 1. 词法分析
        let tokens = lexer::lex(source)?;
        
        // 调试：打印所有token
        #[cfg(debug_assertions)]
        {
            println!("Tokens:");
            for (i, t) in tokens.iter().enumerate() {
                println!("  {}: {:?} at {}", i, t.token, t.loc);
            }
            println!();
        }
        
        // 2. 语法分析
        let ast = parser::parse(tokens)?;
        
        // 3. 语义分析
        let mut analyzer = semantic::SemanticAnalyzer::new();
        analyzer.analyze(&ast)?;

        // 4. 代码生成 - 生成LLVM IR（字符串常量已在生成器内处理）
        let mut ir_gen = codegen::IRGenerator::new();
        // 传递多平台配置
        ir_gen.set_platform_config(&self.options);
        // 传递类型注册表以支持正确的方法名生成
        ir_gen.set_type_registry(analyzer.get_type_registry().clone());
        let mut ir = ir_gen.generate(&ast)?;
        
        // 5. 如果启用了混淆，应用IR混淆
        if self.options.obfuscate {
            use codegen::obfuscator::IRObfuscator;
            let mut obfuscator = IRObfuscator::new();
            ir = obfuscator.obfuscate_ir(&ir);
        }
        
        // 输出到文件
        std::fs::write(output_path, ir)
            .map_err(|e| error::cayError::Io(e.to_string()))?;
        
        Ok(())
    }

    /// 从文件编译，自动执行预处理
    /// 
    /// # Arguments
    /// * `input_path` - 输入源文件路径
    /// * `output_path` - 输出 LLVM IR 文件路径
    /// 
    /// # Returns
    /// 编译成功返回 Ok(())
    pub fn compile_file(&self, input_path: &str, output_path: &str) -> cayResult<()> {
        // 读取源文件
        let source = std::fs::read_to_string(input_path)
            .map_err(|e| error::cayError::Io(
                format!("无法读取源文件 '{}': {}", input_path, e)
            ))?;
        
        // 获取基础目录（用于解析相对路径的 #include）
        let base_dir = Path::new(input_path)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        
        // 预处理
        let preprocessed = preprocessor::preprocess(&source, input_path, base_dir)?;
        
        // 编译预处理后的代码
        self.compile(&preprocessed, output_path)
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_lexer() {
        let source = r#"public class hello {
    public static void main() {
        print("Hello, World");
    }
}"#;
        let tokens = lexer::lex(source).unwrap();
        println!("Tokens:");
        for (i, t) in tokens.iter().enumerate() {
            println!("  {}: {:?} at {}", i, t.token, t.loc);
        }
    }

    #[test]
    fn test_hello_parser() {
        let source = r#"public class hello {
    public static void main() {
        print("Hello, World");
    }
}"#;
        let tokens = lexer::lex(source).unwrap();
        let ast = parser::parse(tokens).unwrap();
        println!("AST: {:?}", ast);
    }

    #[test]
    fn test_preprocessor_define() {
        let source = r#"
#define DEBUG 1
public class Test {
    public static void main() {
        int x = DEBUG;
    }
}
"#;
        let preprocessed = preprocessor::preprocess(source, "test.cay", ".").unwrap();
        assert!(preprocessed.contains("int x = 1;"));
    }

    #[test]
    fn test_preprocessor_ifdef() {
        let source = r#"
#define DEBUG
#ifdef DEBUG
public class DebugClass {
}
#endif
"#;
        let preprocessed = preprocessor::preprocess(source, "test.cay", ".").unwrap();
        assert!(preprocessed.contains("DebugClass"));
    }
}