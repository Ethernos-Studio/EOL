use std::path::{Path, PathBuf};
use anyhow::{Result, Context, bail};
use crate::cavly::config::{CavlyConfig, default_config_template, default_lib_config_template, ProjectType};
use crate::cavly::{CONFIG_FILE, ensure_dir};

/// Cavly 项目管理器
pub struct Project;

impl Project {
    /// 初始化新项目
    /// 
    /// # 参数
    /// - `path`: 项目目录路径
    /// - `name`: 项目名称（可选，默认使用目录名）
    /// - `project_type`: 项目类型（bin 或 lib）
    /// 
    /// # 复杂度
    /// - 时间: O(1)
    /// - 空间: O(1)
    pub fn init(path: &Path, name: Option<&str>, project_type: ProjectType) -> Result<()> {
        let project_name = name.map(String::from)
            .or_else(|| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(String::from)
            })
            .unwrap_or_else(|| "my-project".to_string());
        
        // 验证项目名称
        Self::validate_name(&project_name)?;
        
        // 创建项目目录
        ensure_dir(path)?;
        
        // 创建 src 目录
        let src_dir = path.join("src");
        ensure_dir(&src_dir)?;
        
        // 创建 cavly.toml
        let config_path = path.join(CONFIG_FILE);
        if config_path.exists() {
            bail!("配置文件已存在: {}", config_path.display());
        }
        
        // 根据项目类型选择模板
        let config_content = match project_type {
            ProjectType::Bin => default_config_template(&project_name),
            ProjectType::Lib => default_lib_config_template(&project_name),
        };
        
        std::fs::write(&config_path, config_content)
            .with_context(|| format!("写入配置文件失败: {}", config_path.display()))?;
        
        // 创建默认源文件
        match project_type {
            ProjectType::Bin => {
                let main_path = src_dir.join("main.cay");
                if !main_path.exists() {
                    let main_content = r#"// Cavvy 主程序入口

public class main {
    public static void main() {
        println("Hello, Cavvy!");
    }
}
"#;
                    std::fs::write(&main_path, main_content)
                        .with_context(|| format!("写入主文件失败: {}", main_path.display()))?;
                }
            }
            ProjectType::Lib => {
                let lib_path = src_dir.join("lib.cay");
                if !lib_path.exists() {
                    let lib_content = format!(r#"// Cavvy 库项目: {}

// 导出模块示例
public class {} {{
    // 公共函数会被导出到库中
    public static int add(int a, int b) {{
        return a + b;
    }}
    
    public static void greet() {{
        println("Hello from {} library!");
    }}
}}
"#, project_name, Self::to_class_name(&project_name), project_name);
                    std::fs::write(&lib_path, lib_content)
                        .with_context(|| format!("写入库文件失败: {}", lib_path.display()))?;
                }
            }
        };
        
        // 创建 .gitignore
        let gitignore_path = path.join(".gitignore");
        if !gitignore_path.exists() {
            let gitignore_content = match project_type {
                ProjectType::Bin => r#"# Cavvy 构建产物
target/
*.exe
*.ll
*.o

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# 操作系统
.DS_Store
Thumbs.db
"#,
                ProjectType::Lib => r#"# Cavvy 构建产物
target/
*.exe
*.ll
*.o
*.lib
*.a
*.dll
*.so
*.dylib

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# 操作系统
.DS_Store
Thumbs.db
"#,
            };
            std::fs::write(&gitignore_path, gitignore_content)
                .with_context(|| format!("写入 .gitignore 失败: {}", gitignore_path.display()))?;
        }
        
        let type_str = match project_type {
            ProjectType::Bin => "可执行项目",
            ProjectType::Lib => "库项目",
        };
        
        let main_file_name = match project_type {
            ProjectType::Bin => "main.cay",
            ProjectType::Lib => "lib.cay",
        };
        
        println!("已在 {} 创建{} '{}'", path.display(), type_str, project_name);
        println!("  配置文件: {}", config_path.display());
        println!("  主文件: {}", src_dir.join(main_file_name).display());
        
        Ok(())
    }
    
    /// 将项目名称转换为类名（首字母大写）
    fn to_class_name(name: &str) -> String {
        let mut result = String::new();
        let mut capitalize = true;
        
        for c in name.chars() {
            if c == '_' || c == '-' {
                capitalize = true;
            } else if capitalize {
                result.push(c.to_ascii_uppercase());
                capitalize = false;
            } else {
                result.push(c);
            }
        }
        
        if result.is_empty() {
            result = "Lib".to_string();
        }
        
        result
    }
    
    /// 验证项目名称有效性
    /// 
    /// 规则:
    /// - 只能包含字母、数字、下划线和连字符
    /// - 不能以数字开头
    /// - 不能为空
    fn validate_name(name: &str) -> Result<()> {
        if name.is_empty() {
            bail!("项目名称不能为空");
        }
        
        let first_char = name.chars().next().unwrap();
        if first_char.is_ascii_digit() {
            bail!("项目名称不能以数字开头");
        }
        
        for c in name.chars() {
            if !c.is_ascii_alphanumeric() && c != '_' && c != '-' {
                bail!("项目名称只能包含字母、数字、下划线和连字符");
            }
        }
        
        Ok(())
    }
    
    /// 检查目录是否为 Cavly 项目
    pub fn is_project(path: &Path) -> bool {
        path.join(CONFIG_FILE).exists()
    }
    
    /// 获取项目信息
    pub fn info(path: &Path) -> Result<ProjectInfo> {
        let config_path = path.join(CONFIG_FILE);
        if !config_path.exists() {
            bail!("当前目录不是 Cavly 项目（找不到 cavly.toml）");
        }
        
        let config = CavlyConfig::from_file(&config_path)?;
        
        // 检查源文件
        let src_dir = path.join(&config.package.src_dir);
        let main_file = src_dir.join(&config.package.main);
        let source_exists = main_file.exists();
        
        // 检查目标目录
        let target_dir = path.join(&config.package.target_dir);
        let has_build = target_dir.exists();
        
        Ok(ProjectInfo {
            name: config.package.name.clone(),
            version: config.package.version.clone(),
            description: config.package.description.clone(),
            authors: config.package.authors.clone(),
            license: config.package.license.clone(),
            main_file,
            source_exists,
            target_dir,
            has_build,
            config,
        })
    }
    
    /// 添加 FFI 库配置
    pub fn add_ffi_lib(path: &Path, name: &str, lib: &str) -> Result<()> {
        let config_path = path.join(CONFIG_FILE);
        let mut config = CavlyConfig::from_file(&config_path)?;
        
        use crate::cavly::config::ExternalLibrary;
        
        let ext_lib = ExternalLibrary {
            name: name.to_string(),
            lib: lib.to_string(),
            static_lib: false,
            deps: Vec::new(),
            path: None,
            platform: std::collections::HashMap::new(),
        };
        
        config.ffi.libraries.insert(name.to_string(), ext_lib);
        config.to_file(&config_path)?;
        
        println!("已添加 FFI 库: {} ({})", name, lib);
        Ok(())
    }
    
    /// 添加系统库
    pub fn add_system_lib(path: &Path, lib: &str) -> Result<()> {
        let config_path = path.join(CONFIG_FILE);
        let mut config = CavlyConfig::from_file(&config_path)?;
        
        if !config.ffi.system_libs.contains(&lib.to_string()) {
            config.ffi.system_libs.push(lib.to_string());
            config.to_file(&config_path)?;
            println!("已添加系统库: {}", lib);
        } else {
            println!("系统库已存在: {}", lib);
        }
        
        Ok(())
    }
}

/// 项目信息
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
    pub license: String,
    pub main_file: PathBuf,
    pub source_exists: bool,
    pub target_dir: PathBuf,
    pub has_build: bool,
    pub config: CavlyConfig,
}

impl ProjectInfo {
    /// 格式化输出项目信息
    pub fn print(&self) {
        println!("项目: {} ({})", self.name, self.version);
        
        if !self.description.is_empty() {
            println!("描述: {}", self.description);
        }
        
        if !self.authors.is_empty() {
            println!("作者: {}", self.authors.join(", "));
        }
        
        if !self.license.is_empty() {
            println!("许可证: {}", self.license);
        }
        
        println!("主文件: {} {}", 
            self.main_file.display(),
            if self.source_exists { "[存在]" } else { "[缺失]" }
        );
        
        println!("目标目录: {} {}",
            self.target_dir.display(),
            if self.has_build { "[有构建产物]" } else { "[空]" }
        );
        
        // FFI 库信息
        if !self.config.ffi.system_libs.is_empty() {
            println!("系统库: {}", self.config.ffi.system_libs.join(", "));
        }
        
        if !self.config.ffi.libraries.is_empty() {
            println!("第三方库:");
            for (name, lib) in &self.config.ffi.libraries {
                println!("  - {} ({})", name, lib.lib);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validate_name_valid() {
        assert!(Project::validate_name("my-project").is_ok());
        assert!(Project::validate_name("my_project").is_ok());
        assert!(Project::validate_name("MyProject123").is_ok());
    }

    #[test]
    fn test_validate_name_invalid() {
        assert!(Project::validate_name("").is_err());
        assert!(Project::validate_name("123project").is_err());
        assert!(Project::validate_name("my project").is_err());
        assert!(Project::validate_name("my@project").is_err());
    }

    #[test]
    fn test_project_init() {
        use crate::cavly::config::ProjectType;
        
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("test-project");
        
        Project::init(&project_path, Some("test-project"), ProjectType::Bin).unwrap();
        
        assert!(project_path.join("cavly.toml").exists());
        assert!(project_path.join("src").exists());
        assert!(project_path.join("src/main.cay").exists());
        assert!(project_path.join(".gitignore").exists());
    }
    
    #[test]
    fn test_lib_project_init() {
        use crate::cavly::config::ProjectType;
        
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("test-lib");
        
        Project::init(&project_path, Some("test-lib"), ProjectType::Lib).unwrap();
        
        assert!(project_path.join("cavly.toml").exists());
        assert!(project_path.join("src").exists());
        assert!(project_path.join("src/lib.cay").exists());
        assert!(project_path.join(".gitignore").exists());
    }

    #[test]
    fn test_is_project() {
        let temp = TempDir::new().unwrap();
        
        // 空目录不是项目
        assert!(!Project::is_project(temp.path()));
        
        // 创建配置文件后才是项目
        std::fs::write(temp.path().join("cavly.toml"), "").unwrap();
        assert!(Project::is_project(temp.path()));
    }

    #[test]
    fn test_project_info() {
        use crate::cavly::config::ProjectType;
        
        let temp = TempDir::new().unwrap();
        Project::init(temp.path(), Some("test"), ProjectType::Bin).unwrap();
        
        let info = Project::info(temp.path()).unwrap();
        assert_eq!(info.name, "test");
        assert!(info.source_exists);
    }
}
