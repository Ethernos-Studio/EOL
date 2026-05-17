use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};

/// 项目类型
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    /// 可执行程序（默认）
    #[default]
    Bin,
    /// 库项目
    Lib,
}

/// Cavly 项目配置（cavly.toml）
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CavlyConfig {
    /// 包信息
    pub package: PackageConfig,
    
    /// 构建配置
    #[serde(default)]
    pub build: BuildConfig,
    
    /// FFI 配置
    #[serde(default)]
    pub ffi: FfiConfig,
    
    /// 依赖配置
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
    
    /// 开发依赖
    #[serde(default)]
    pub dev_dependencies: HashMap<String, Dependency>,
    
    /// 工作区配置（库搜索路径、本地库项目）
    #[serde(default)]
    pub workspace: WorkspaceConfig,
    
    /// 库项目特定配置
    #[serde(default)]
    pub lib: LibConfig,
    
    /// 自定义配置段
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}

/// 包信息配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageConfig {
    /// 包名
    pub name: String,
    
    /// 版本
    pub version: String,
    
    /// 项目类型: bin 或 lib
    #[serde(default)]
    pub project_type: ProjectType,
    
    /// 描述
    #[serde(default)]
    pub description: String,
    
    /// 作者
    #[serde(default)]
    pub authors: Vec<String>,
    
    /// 许可证
    #[serde(default)]
    pub license: String,
    
    /// 主入口文件
    #[serde(default = "default_main")]
    pub main: String,
    
    /// 源代码目录
    #[serde(default = "default_src_dir")]
    pub src_dir: String,
    
    /// 输出目录
    #[serde(default = "default_target_dir")]
    pub target_dir: String,
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: "0.1.0".to_string(),
            project_type: ProjectType::default(),
            description: String::new(),
            authors: Vec::new(),
            license: String::new(),
            main: default_main(),
            src_dir: default_src_dir(),
            target_dir: default_target_dir(),
        }
    }
}

fn default_main() -> String {
    "main.cay".to_string()
}

fn default_src_dir() -> String {
    "src".to_string()
}

fn default_target_dir() -> String {
    "target".to_string()
}

/// 构建配置
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct BuildConfig {
    /// 优化级别: 0, 1, 2, 3, s, z
    #[serde(default = "default_opt_level")]
    pub opt_level: String,
    
    /// 是否启用调试信息
    #[serde(default)]
    pub debug: bool,
    
    /// 是否静态链接
    #[serde(default)]
    pub static_link: bool,
    
    /// 目标平台
    #[serde(default)]
    pub target: Option<String>,
    
    /// 额外的编译器标志
    #[serde(default)]
    pub cflags: Vec<String>,
    
    /// 额外的链接器标志
    #[serde(default)]
    pub ldflags: Vec<String>,
    
    /// 库搜索路径
    #[serde(default)]
    pub lib_paths: Vec<String>,
    
    /// 要链接的库
    #[serde(default)]
    pub libs: Vec<String>,
    
    /// 启用 LTO
    #[serde(default)]
    pub lto: bool,
    
    /// 启用 IR 优化
    #[serde(default)]
    pub opt_ir: bool,
    
    /// 保留 IR 文件
    #[serde(default)]
    pub keep_ir: bool,
    
    /// 输出文件名
    #[serde(default)]
    pub output_name: Option<String>,
}

fn default_opt_level() -> String {
    "2".to_string()
}

/// FFI 配置
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct FfiConfig {
    /// 系统库链接
    #[serde(default)]
    pub system_libs: Vec<String>,
    
    /// 第三方库配置
    #[serde(default)]
    pub libraries: HashMap<String, ExternalLibrary>,
    
    /// 头文件路径（用于 FFI 声明生成）
    #[serde(default)]
    pub include_paths: Vec<String>,
    
    /// 链接器脚本
    #[serde(default)]
    pub linker_script: Option<String>,
    
    /// 自定义链接选项
    #[serde(default)]
    pub link_options: Vec<String>,
}

/// 外部库配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExternalLibrary {
    /// 库名
    pub name: String,
    
    /// 库文件路径或名称
    pub lib: String,
    
    /// 库搜索路径
    #[serde(default)]
    pub path: Option<String>,
    
    /// 是否静态链接
    #[serde(default)]
    pub static_lib: bool,
    
    /// 依赖的其他库
    #[serde(default)]
    pub deps: Vec<String>,
    
    /// 平台特定配置
    #[serde(default)]
    pub platform: HashMap<String, PlatformConfig>,
}

impl Default for ExternalLibrary {
    fn default() -> Self {
        Self {
            name: String::new(),
            lib: String::new(),
            path: None,
            static_lib: false,
            deps: Vec::new(),
            platform: HashMap::new(),
        }
    }
}

/// 工作区配置
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct WorkspaceConfig {
    /// 本地库项目路径列表
    /// 这些路径会被搜索以解析本地依赖
    #[serde(default)]
    pub members: Vec<String>,
    
    /// 额外的库搜索路径
    /// 在解析依赖时会搜索这些路径
    #[serde(default)]
    pub lib_paths: Vec<String>,
    
    /// 默认构建配置（会被成员项目继承）
    #[serde(default)]
    pub default_build: Option<BuildConfig>,
    
    /// 默认 FFI 配置（会被成员项目继承）
    #[serde(default)]
    pub default_ffi: Option<FfiConfig>,
}

/// 库项目配置
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LibConfig {
    /// 库类型: static（静态库）, dynamic（动态库）
    #[serde(default = "default_lib_type")]
    pub lib_type: String,
    
    /// 导出的模块列表（为空则导出所有 public）
    #[serde(default)]
    pub exports: Vec<String>,
    
    /// 头文件生成配置
    #[serde(default)]
    pub header: HeaderConfig,
    
    /// 安装路径（相对于 target）
    #[serde(default = "default_install_path")]
    pub install_path: String,

    /// 只做 Cavvy→IR 检查，供下游项目导入
    #[serde(default)]
    pub only_include: bool,
}

fn default_lib_type() -> String {
    "static".to_string()
}

fn default_install_path() -> String {
    "lib".to_string()
}

/// 头文件生成配置
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct HeaderConfig {
    /// 是否生成 C 头文件
    #[serde(default)]
    pub generate: bool,
    
    /// 头文件名
    #[serde(default)]
    pub name: Option<String>,
    
    /// 头文件包含路径前缀
    #[serde(default)]
    pub include_prefix: String,
}

/// 平台特定配置
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PlatformConfig {
    /// 库名（平台特定）
    #[serde(default)]
    pub lib: Option<String>,
    
    /// 库路径（平台特定）
    #[serde(default)]
    pub path: Option<String>,
    
    /// 额外链接选项
    #[serde(default)]
    pub ldflags: Vec<String>,
}

/// 依赖配置
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Dependency {
    /// 简单版本指定
    Simple(String),
    /// 详细配置
    Detailed(DetailedDependency),
}

/// 详细依赖配置
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DetailedDependency {
    /// 版本要求
    pub version: Option<String>,
    
    /// Git 仓库地址
    pub git: Option<String>,
    
    /// Git 分支
    pub branch: Option<String>,
    
    /// Git 标签
    pub tag: Option<String>,
    
    /// 本地路径
    pub path: Option<PathBuf>,
    
    /// 是否可选
    #[serde(default)]
    pub optional: bool,
    
    /// 目标平台特定
    #[serde(default)]
    pub target: Option<String>,
}

impl CavlyConfig {
    /// 从文件加载配置
    /// 
    /// # 复杂度
    /// - 时间: O(n)，n 为文件大小
    /// - 空间: O(1) 额外空间
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("读取配置文件失败: {}", path.display()))?;
        
        let config: CavlyConfig = toml::from_str(&content)
            .with_context(|| format!("解析配置文件失败: {}", path.display()))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// 保存配置到文件
    /// 
    /// # 复杂度
    /// - 时间: O(n)，n 为配置大小
    /// - 空间: O(1) 额外空间
    pub fn to_file(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("序列化配置失败")?;
        
        std::fs::write(path, content)
            .with_context(|| format!("写入配置文件失败: {}", path.display()))?;
        
        Ok(())
    }
    
    /// 验证配置有效性
    fn validate(&self) -> Result<()> {
        if self.package.name.is_empty() {
            anyhow::bail!("包名不能为空");
        }
        
        if self.package.version.is_empty() {
            anyhow::bail!("版本号不能为空");
        }
        
        Ok(())
    }
    
    /// 获取主源文件完整路径
    pub fn main_source_path(&self, project_root: &Path) -> PathBuf {
        project_root.join(&self.package.src_dir).join(&self.package.main)
    }
    
    /// 获取目标目录路径
    pub fn target_path(&self, project_root: &Path) -> PathBuf {
        project_root.join(&self.package.target_dir)
    }
    
    /// 获取输出文件名
    pub fn output_filename(&self) -> String {
        self.build.output_name.clone()
            .unwrap_or_else(|| self.package.name.clone())
    }
    
    /// 获取所有库搜索路径（包括 FFI）
    pub fn all_lib_paths(&self) -> Vec<String> {
        let mut paths = self.build.lib_paths.clone();
        
        for lib in self.ffi.libraries.values() {
            if let Some(ref path) = lib.path {
                paths.push(path.clone());
            }
            
            // 添加平台特定路径
            #[cfg(target_os = "windows")]
            let platform_key = "windows";
            #[cfg(target_os = "linux")]
            let platform_key = "linux";
            #[cfg(target_os = "macos")]
            let platform_key = "macos";
            
            if let Some(platform) = lib.platform.get(platform_key) {
                if let Some(ref path) = platform.path {
                    paths.push(path.clone());
                }
            }
        }
        
        paths
    }
    
    /// 获取所有要链接的库
    pub fn all_libs(&self) -> Vec<String> {
        let mut libs = self.build.libs.clone();
        libs.extend(self.ffi.system_libs.clone());
        
        for lib in self.ffi.libraries.values() {
            libs.push(lib.lib.clone());
            libs.extend(lib.deps.clone());
        }
        
        libs
    }
    
    /// 获取优化级别参数
    pub fn opt_flag(&self) -> String {
        format!("-O{}", self.build.opt_level)
    }
    
    /// 合并另一个配置（用于继承工作区或依赖库的配置）
    /// 
    /// # 说明
    /// - 当前配置优先级更高，不会被覆盖
    /// - 合并 build: lib_paths, libs, cflags, ldflags 会追加
    /// - 合并 ffi: system_libs, include_paths, link_options 会追加
    /// 
    /// # 复杂度
    /// - 时间: O(n + m)，n 和 m 为配置项数量
    /// - 空间: O(1) 额外空间
    pub fn merge(&mut self, other: &CavlyConfig) {
        // 合并 build 配置
        self.build.lib_paths.extend(other.build.lib_paths.clone());
        self.build.libs.extend(other.build.libs.clone());
        self.build.cflags.extend(other.build.cflags.clone());
        self.build.ldflags.extend(other.build.ldflags.clone());
        
        // 如果当前没有设置某些选项，继承其他配置的
        if self.build.opt_level == default_opt_level() && other.build.opt_level != default_opt_level() {
            self.build.opt_level = other.build.opt_level.clone();
        }
        if !self.build.debug && other.build.debug {
            self.build.debug = other.build.debug;
        }
        if !self.build.static_link && other.build.static_link {
            self.build.static_link = other.build.static_link;
        }
        if self.build.target.is_none() && other.build.target.is_some() {
            self.build.target = other.build.target.clone();
        }
        if !self.build.lto && other.build.lto {
            self.build.lto = other.build.lto;
        }
        
        // 合并 FFI 配置
        self.ffi.system_libs.extend(other.ffi.system_libs.clone());
        self.ffi.include_paths.extend(other.ffi.include_paths.clone());
        self.ffi.link_options.extend(other.ffi.link_options.clone());
        
        // 合并 FFI 库配置
        for (name, lib) in &other.ffi.libraries {
            self.ffi.libraries.entry(name.clone()).or_insert_with(|| lib.clone());
        }
        
        // 合并工作区 lib_paths
        self.workspace.lib_paths.extend(other.workspace.lib_paths.clone());
    }
    
    /// 获取库输出文件名（仅用于 lib 类型项目）
    pub fn lib_output_filename(&self) -> String {
        let base_name = self.build.output_name.clone()
            .unwrap_or_else(|| self.package.name.clone());
        
        match self.lib.lib_type.as_str() {
            "static" => {
                if cfg!(target_os = "windows") {
                    format!("{}.lib", base_name)
                } else {
                    format!("lib{}.a", base_name)
                }
            }
            "dynamic" => {
                if cfg!(target_os = "windows") {
                    format!("{}.dll", base_name)
                } else if cfg!(target_os = "macos") {
                    format!("lib{}.dylib", base_name)
                } else {
                    format!("lib{}.so", base_name)
                }
            }
            _ => base_name,
        }
    }
    
    /// 获取库安装路径（相对于 target）
    pub fn lib_install_path(&self, project_root: &Path) -> PathBuf {
        let target = self.target_path(project_root);
        target.join(&self.lib.install_path)
    }
    
    /// 检查是否为库项目
    pub fn is_lib(&self) -> bool {
        self.package.project_type == ProjectType::Lib
    }
    
    /// 检查是否为可执行项目
    pub fn is_bin(&self) -> bool {
        self.package.project_type == ProjectType::Bin
    }
}

/// 创建默认可执行项目配置模板
/// 
/// # 复杂度
/// - 时间: O(1)
/// - 空间: O(1)
pub fn default_config_template(name: &str) -> String {
    format!(r#"[package]
name = "{}"
version = "0.1.0"
description = "A Cavvy project"
authors = []
license = "MIT"
# project_type = "bin"  # bin（可执行）或 lib（库）
main = "main.cay"
src_dir = "src"
target_dir = "target"

[build]
opt_level = "2"
debug = false
static_link = false
# target = "x86_64-w64-mingw32"
cflags = []
ldflags = []
lib_paths = []
libs = []
lto = false
opt_ir = false
keep_ir = false

[ffi]
# 系统库，如 "user32", "kernel32" (Windows) 或 "m", "pthread" (Linux)
system_libs = []
include_paths = []
link_options = []

# 第三方库配置示例
# [ffi.libraries.sdl2]
# name = "SDL2"
# lib = "SDL2"
# path = "./lib"
# static_lib = false
# deps = ["SDL2main"]
#
# [ffi.libraries.sdl2.platform.windows]
# lib = "SDL2"
# path = "C:/SDL2/lib"
# ldflags = ["-lSDL2main", "-lSDL2"]
#
# [ffi.libraries.sdl2.platform.linux]
# lib = "SDL2"
# path = "/usr/lib/x86_64-linux-gnu"

[workspace]
# 本地库项目路径列表
# members = ["../mylib", "./libs/helper"]
# 额外的库搜索路径
# lib_paths = ["./lib", "/usr/local/lib"]

[dependencies]
# 依赖其他 Cavvy 包
# example = "1.0.0"
# mylib = {{ git = "https://github.com/user/mylib", branch = "main" }}
# local = {{ path = "../local" }}

[dev-dependencies]
# 仅开发时使用的依赖
"#, name)
}

/// 创建默认库项目配置模板
/// 
/// # 复杂度
/// - 时间: O(1)
/// - 空间: O(1)
pub fn default_lib_config_template(name: &str) -> String {
    format!(r#"[package]
name = "{}"
version = "0.1.0"
description = "A Cavvy library"
authors = []
license = "MIT"
project_type = "lib"
main = "lib.cay"
src_dir = "src"
target_dir = "target"

[build]
opt_level = "2"
debug = false
static_link = true
cflags = []
ldflags = []
lib_paths = []
libs = []
lto = false
opt_ir = false
keep_ir = false

[lib]
# 库类型: static（静态库）或 dynamic（动态库）
lib_type = "static"
# 导出的模块列表（为空则导出所有 public）
exports = []
# 安装路径（相对于 target）
install_path = "lib"
# 仅接口模式：只做 Cavvy→IR 检查，不编译/不链接成库，供下游项目导入
# only_include = true

[lib.header]
# 是否生成 C 头文件
generate = true
# 头文件名（默认为库名.h）
# name = "mylib.h"
# 头文件包含路径前缀
include_prefix = ""

[ffi]
system_libs = []
include_paths = []
link_options = []

[workspace]
# 额外的库搜索路径
# lib_paths = []

[dependencies]

[dev-dependencies]
"#, name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_from_file() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("cavly.toml");
        
        let toml_content = r#"
[package]
name = "test-project"
version = "1.0.0"
description = "Test project"

[build]
opt_level = "3"
debug = true

[ffi]
system_libs = ["m", "pthread"]

[ffi.libraries.testlib]
name = "TestLib"
lib = "test"
static_lib = true
"#;
        
        std::fs::write(&config_path, toml_content).unwrap();
        
        let config = CavlyConfig::from_file(&config_path).unwrap();
        assert_eq!(config.package.name, "test-project");
        assert_eq!(config.build.opt_level, "3");
        assert!(config.build.debug);
        assert_eq!(config.ffi.system_libs, vec!["m", "pthread"]);
    }

    #[test]
    fn test_config_validation_empty_name() {
        let config = CavlyConfig {
            package: PackageConfig {
                name: "".to_string(),
                version: "1.0.0".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_all_libs() {
        let mut config = CavlyConfig::default();
        config.build.libs = vec!["a".to_string()];
        config.ffi.system_libs = vec!["b".to_string()];
        config.ffi.libraries.insert("c".to_string(), ExternalLibrary {
            name: "C".to_string(),
            lib: "c".to_string(),
            static_lib: false,
            deps: vec!["d".to_string()],
            ..Default::default()
        });
        
        let libs = config.all_libs();
        assert!(libs.contains(&"a".to_string()));
        assert!(libs.contains(&"b".to_string()));
        assert!(libs.contains(&"c".to_string()));
        assert!(libs.contains(&"d".to_string()));
    }

    #[test]
    fn test_opt_flag() {
        let mut config = CavlyConfig::default();
        config.build.opt_level = "3".to_string();
        assert_eq!(config.opt_flag(), "-O3");
    }
}
