use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, Context, bail};
use crate::cavly::config::{CavlyConfig, ProjectType};
use crate::cavly::workspace::{WorkspaceResolver, ResolvedDependency, topological_sort};
use crate::cavly::{ensure_dir, TARGET_DIR};

/// 构建器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildState {
    Idle,
    Compiling,
    Linking,
    Complete,
    Failed,
}

/// Cavly 构建器
/// 
/// 通过调用 cayc 编译器实现构建，确保与直接调用 cayc 的行为一致
pub struct Builder {
    /// 项目根目录
    project_root: PathBuf,
    /// 构建配置
    config: CavlyConfig,
    /// 当前状态
    state: BuildState,
    /// 是否 verbose 模式
    verbose: bool,
    /// 解析后的依赖列表
    dependencies: Vec<ResolvedDependency>,
}

impl Builder {
    /// 创建新的构建器
    /// 
    /// # 复杂度
    /// - 时间: O(1)
    /// - 空间: O(1)
    pub fn new(project_root: PathBuf, config: CavlyConfig) -> Self {
        Self {
            project_root,
            config,
            state: BuildState::Idle,
            verbose: false,
            dependencies: Vec::new(),
        }
    }
    
    /// 创建新的构建器并解析依赖
    /// 
    /// # 复杂度
    /// - 时间: O(n*m)，n 为依赖数量，m 为每个依赖的配置大小
    /// - 空间: O(n)
    pub fn with_dependencies(project_root: PathBuf, mut config: CavlyConfig) -> Result<Self> {
        let mut resolver = WorkspaceResolver::new(project_root.clone());
        
        // 解析所有依赖
        let dependencies = resolver.resolve_all(&config)?;
        
        // 拓扑排序依赖（确保被依赖的先构建）
        let sorted_deps = topological_sort(&dependencies)?;
        
        // 合并所有依赖的配置到主配置
        resolver.merge_dependencies_config(&mut config, &sorted_deps);
        
        Ok(Self {
            project_root,
            config,
            state: BuildState::Idle,
            verbose: false,
            dependencies: sorted_deps,
        })
    }
    
    /// 设置 verbose 模式
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
    
    /// 获取当前状态
    pub fn state(&self) -> BuildState {
        self.state
    }
    
    /// 执行完整构建流程
    /// 
    /// # 流程
    /// 1. 首先构建所有依赖库（如果是 lib 项目）
    /// 2. 验证源文件存在
    /// 3. 查找 cayc 编译器
    /// 4. 构建 cayc 命令行参数
    /// 5. 调用 cayc 执行编译
    /// 
    /// # 复杂度
    /// - 时间: O(n + m)，n 为源码大小，m 为链接复杂度
    /// - 空间: O(n) 临时文件
    pub fn build(&mut self) -> Result<PathBuf> {
        self.state = BuildState::Compiling;
        
        // 1. 先构建所有依赖库
        self.build_dependencies()?;
        
        // 2. 验证源文件
        let source_path = self.config.main_source_path(&self.project_root);
        if !source_path.exists() {
            bail!("主源文件不存在: {}", source_path.display());
        }
        
        // 3. 准备目标目录
        let target_dir = self.config.target_path(&self.project_root);
        ensure_dir(&target_dir)?;
        
        // 4. 查找 cayc 编译器
        let cayc_path = find_cayc()?;
        
        // 5. 确定输出文件路径
        let output_path = self.determine_output_path(&target_dir)?;
        
        // 6. 构建 cayc 命令行参数
        let args = self.build_cayc_args(&source_path, &output_path)?;
        
        if self.verbose {
            if self.config.is_lib() && self.config.lib.only_include {
                println!("Cavly: 项目: {} v{} (库 - 仅接口/only_include)", 
                    self.config.package.name, 
                    self.config.package.version
                );
            } else {
                println!("Cavly: 项目: {} v{} ({})", 
                    self.config.package.name, 
                    self.config.package.version,
                    if self.config.is_lib() { "库" } else { "可执行" }
                );
            }
            println!("Cavly: 调用: {} {}", 
                cayc_path.display(),
                args.join(" ")
            );
        }

        // 7. 执行 cayc 编译
        self.state = BuildState::Linking;
        
        let output = Command::new(&cayc_path)
            .args(&args)
            .current_dir(&self.project_root)
            .output()
            .with_context(|| format!("执行 cayc 失败: {}", cayc_path.display()))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            bail!("编译失败:\nstdout:\n{}\nstderr:\n{}", stdout, stderr);
        }
        
        // 8. 检查输出文件是否生成
        if !output_path.exists() {
            bail!("编译未生成输出文件: {}", output_path.display());
        }
        
        // 9. 如果是库项目且不是 only_include，安装到 lib 目录
        if self.config.is_lib() && !self.config.lib.only_include {
            self.install_library(&output_path)?;
        }
        
        self.state = BuildState::Complete;
        
        if self.verbose {
            println!("Cavly: 构建成功: {}", output_path.display());
        }
        
        Ok(output_path)
    }
    
    /// 构建所有依赖库
    /// 
    /// # 复杂度
    /// - 时间: O(n*m)，n 为依赖数量，m 为每个依赖的构建时间
    /// - 空间: O(n)
    fn build_dependencies(&mut self) -> Result<()> {
        if self.dependencies.is_empty() {
            return Ok(());
        }

        if self.verbose {
            println!("Cavly: 开始构建 {} 个依赖...", self.dependencies.len());
        }

        for dep in &self.dependencies {
            // 跳过 only_include 依赖：它们只做接口检查，不产出 .lib
            if dep.config.lib.only_include {
                if self.verbose {
                    println!("Cavly: 跳过 only_include 依赖: {}", dep.name);
                }
                continue;
            }

            if self.verbose {
                println!("Cavly: 构建依赖: {} @ {}", dep.name, dep.path.display());
            }

            // 为每个依赖创建构建器
            let mut dep_builder = Builder::new(dep.path.clone(), dep.config.clone())
                .verbose(self.verbose);

            dep_builder.build()?;
        }

        if self.verbose {
            println!("Cavly: 依赖构建完成");
        }

        Ok(())
    }

    /// 确定输出文件路径
    /// 
    /// # 复杂度
    /// - 时间: O(1)
    /// - 空间: O(1)
    fn determine_output_path(&self, target_dir: &Path) -> Result<PathBuf> {
        match self.config.package.project_type {
            ProjectType::Bin => {
                let output_name = self.config.output_filename();
                if self.is_windows_target() {
                    Ok(target_dir.join(format!("{}.exe", output_name)))
                } else {
                    Ok(target_dir.join(&output_name))
                }
            }
            ProjectType::Lib => {
                if self.config.lib.only_include {
                    // only_include 模式：只生成 IR 文件，不链接成库
                    let ir_dir = target_dir.join("ir");
                    ensure_dir(&ir_dir)?;
                    let ir_name = format!("{}.ll", self.config.output_filename());
                    Ok(ir_dir.join(ir_name))
                } else {
                    // 库项目输出到 target/lib 目录
                    let lib_dir = self.config.lib_install_path(&self.project_root);
                    ensure_dir(&lib_dir)?;

                    let lib_filename = self.config.lib_output_filename();
                    Ok(lib_dir.join(lib_filename))
                }
            }
        }
    }
    
    /// 安装库文件
    /// 
    /// # 复杂度
    /// - 时间: O(1)
    /// - 空间: O(1)
    fn install_library(&self, output_path: &Path) -> Result<()> {
        // only_include 模式不产出库文件，无需安装
        if self.config.lib.only_include {
            return Ok(());
        }

        let lib_dir = self.config.lib_install_path(&self.project_root);
        ensure_dir(&lib_dir)?;
        
        // 复制库文件到安装目录
        let lib_filename = self.config.lib_output_filename();
        let install_path = lib_dir.join(&lib_filename);
        
        if output_path != install_path {
            std::fs::copy(output_path, &install_path)
                .with_context(|| format!("安装库文件失败: {} -> {}", 
                    output_path.display(), install_path.display()))?;
        }
        
        // TODO: 生成头文件（如果配置了）
        if self.config.lib.header.generate {
            self.generate_header(&lib_dir)?;
        }
        
        if self.verbose {
            println!("Cavly: 库已安装到: {}", lib_dir.display());
        }
        
        Ok(())
    }
    
    /// 生成 C 头文件
    /// 
    /// # 复杂度
    /// - 时间: O(n)，n 为导出的符号数量
    /// - 空间: O(n)
    fn generate_header(&self, lib_dir: &Path) -> Result<()> {
        let header_name = self.config.lib.header.name.clone()
            .unwrap_or_else(|| format!("{}.h", self.config.package.name));
        
        let header_path = lib_dir.join(&header_name);
        
        // TODO: 解析源文件并生成头文件
        // 目前生成一个占位头文件
        let header_content = format!(r#"/* Cavvy Library Header - Auto Generated */
#ifndef {}_H
#define {}_H

/* TODO: 解析并导出 Cavvy 公共接口 */

#endif /* {}_H */
"#, 
            self.config.package.name.to_uppercase(),
            self.config.package.name.to_uppercase(),
            self.config.package.name.to_uppercase()
        );
        
        std::fs::write(&header_path, header_content)
            .with_context(|| format!("写入头文件失败: {}", header_path.display()))?;
        
        if self.verbose {
            println!("Cavly: 头文件已生成: {}", header_path.display());
        }
        
        Ok(())
    }
    
    /// 构建 cayc 命令行参数
    /// 
    /// # 复杂度
    /// - 时间: O(n)，n 为配置参数数量
    /// - 空间: O(n)
    fn build_cayc_args(&self, source_path: &Path, output_path: &Path) -> Result<Vec<String>> {
        let mut args = Vec::new();

        // only_include 模式：只编译检查，不链接任何库
        let is_only_include = self.config.is_lib() && self.config.lib.only_include;

        // 优化级别
        args.push(self.config.opt_flag());
        
        // 调试信息
        if self.config.build.debug {
            args.push("-g".to_string());
        }
        
        // 静态链接（only_include 模式不需要）
        if !is_only_include && self.config.build.static_link {
            args.push("--static".to_string());
        }
        
        // LTO
        if self.config.build.lto {
            if self.config.build.opt_ir {
                // thin LTO
                args.push("--lto=thin".to_string());
            } else {
                args.push("--lto=full".to_string());
            }
        }
        
        // 目标平台
        if let Some(ref target) = self.config.build.target {
            args.push("--target".to_string());
            args.push(target.clone());
        }
        
        // IR 优化
        if self.config.build.opt_ir {
            args.push("--opt-ir".to_string());
        }
        
        // 保留 IR
        if self.config.build.keep_ir {
            args.push("--keep-ir".to_string());
        }
        
        // only_include 模式不链接任何库（包括依赖库和 FFI 库）
        if !is_only_include {
            // 添加依赖库的搜索路径（跳过 only_include 依赖，它们不产 .lib）
            for dep in &self.dependencies {
                if dep.config.lib.only_include {
                    continue;
                }
                let lib_path = dep.config.lib_install_path(&dep.path);
                if lib_path.exists() {
                    args.push(format!("-L{}", lib_path.display()));
                }
            }

            // 库搜索路径（包括依赖的）
            for path in self.config.all_lib_paths() {
                args.push(format!("-L{}", path));
            }

            // 链接依赖库（跳过 only_include 的依赖，它们没有 .lib 产物）
            for dep in &self.dependencies {
                if dep.config.lib.only_include {
                    continue;
                }
                let lib_name = dep.config.output_filename();
                args.push(format!("-l{}", lib_name));
            }

            // 链接的库（包括 FFI 库）
            for lib in self.config.all_libs() {
                args.push(format!("-l{}", lib));
            }
        }

        // 添加依赖的源代码目录作为包含路径（供 #include 使用）
        for dep in &self.dependencies {
            let dep_src = dep.path.join(&dep.config.package.src_dir);
            if dep_src.exists() {
                args.push(format!("-I{}", dep_src.display()));
            }
        }

        // 额外的 cflags
        if !self.config.build.cflags.is_empty() {
            args.push("--cflags".to_string());
            args.push(self.config.build.cflags.join(" "));
        }
        
        // 额外的 ldflags
        if !self.config.build.ldflags.is_empty() {
            args.push("--ldflags".to_string());
            args.push(self.config.build.ldflags.join(" "));
        }
        
        // 输入文件（相对于项目根目录的路径）
        args.push(source_path.to_string_lossy().to_string());
        
        // 输出文件
        args.push(output_path.to_string_lossy().to_string());
        
        Ok(args)
    }
    
    /// 检查是否为 Windows 目标
    fn is_windows_target(&self) -> bool {
        if let Some(ref target) = self.config.build.target {
            target.contains("windows") || target.contains("mingw")
        } else {
            cfg!(target_os = "windows")
        }
    }
    
    /// 清理构建产物
    /// 
    /// # 复杂度
    /// - 时间: O(1)
    /// - 空间: O(1)
    pub fn clean(&self) -> Result<()> {
        let target_dir = self.config.target_path(&self.project_root);
        
        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir)
                .with_context(|| format!("清理目标目录失败: {}", target_dir.display()))?;
        }
        
        if self.verbose {
            println!("Cavly: 已清理: {}", target_dir.display());
        }
        
        Ok(())
    }
}

/// 查找 cayc 编译器
/// 
/// 搜索顺序:
/// 1. 系统 PATH 中的 cayc
/// 2. 当前可执行文件所在目录下的 cayc
fn find_cayc() -> Result<PathBuf> {
    // 1. 尝试系统 PATH
    if let Ok(output) = Command::new("cayc").arg("--version").output() {
        if output.status.success() {
            return Ok(PathBuf::from("cayc"));
        }
    }
    
    // 2. 尝试当前可执行文件所在目录
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let cayc_exe = if cfg!(target_os = "windows") {
                exe_dir.join("cayc.exe")
            } else {
                exe_dir.join("cayc")
            };
            
            if cayc_exe.exists() {
                return Ok(cayc_exe);
            }
        }
    }
    
    // 3. 尝试当前工作目录下的 target/debug 或 target/release
    if let Ok(cwd) = std::env::current_dir() {
        for profile in &["debug", "release"] {
            let cayc_exe = if cfg!(target_os = "windows") {
                cwd.join("target").join(profile).join("cayc.exe")
            } else {
                cwd.join("target").join(profile).join("cayc")
            };
            
            if cayc_exe.exists() {
                return Ok(cayc_exe);
            }
        }
    }
    
    bail!("找不到 cayc 编译器。请确保 cayc 已安装并在 PATH 中，或与 cavly 在同一目录下")
}

/// 快速构建入口
/// 
/// # 复杂度
/// - 时间: O(n + m)
/// - 空间: O(n)
pub fn quick_build(project_root: &Path, verbose: bool) -> Result<PathBuf> {
    let config_path = project_root.join("cavly.toml");
    let config = CavlyConfig::from_file(&config_path)?;
    
    let mut builder = Builder::new(project_root.to_path_buf(), config)
        .verbose(verbose);
    
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cavly::config::{PackageConfig, BuildConfig};
    use tempfile::TempDir;

    fn create_test_config() -> CavlyConfig {
        CavlyConfig {
            package: PackageConfig {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                main: "main.cay".to_string(),
                src_dir: "src".to_string(),
                target_dir: "target".to_string(),
                ..Default::default()
            },
            build: BuildConfig {
                opt_level: "0".to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_builder_state() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config();
        let builder = Builder::new(temp.path().to_path_buf(), config);
        
        assert_eq!(builder.state(), BuildState::Idle);
    }

    #[test]
    fn test_is_windows_target() {
        let temp = TempDir::new().unwrap();
        let mut config = create_test_config();
        
        // 显式 Windows 目标
        config.build.target = Some("x86_64-w64-mingw32".to_string());
        let builder = Builder::new(temp.path().to_path_buf(), config.clone());
        assert!(builder.is_windows_target());
        
        // Linux 目标
        config.build.target = Some("x86_64-unknown-linux-gnu".to_string());
        let builder = Builder::new(temp.path().to_path_buf(), config);
        assert!(!builder.is_windows_target());
    }

    #[test]
    fn test_builder_verbose() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config();
        let builder = Builder::new(temp.path().to_path_buf(), config)
            .verbose(true);
        
        assert!(builder.verbose);
    }

    #[test]
    fn test_build_cayc_args() {
        let temp = TempDir::new().unwrap();
        let mut config = create_test_config();
        
        // 设置一些构建选项
        config.build.debug = true;
        config.build.static_link = true;
        config.build.opt_level = "3".to_string();
        config.build.libs = vec!["m".to_string()];
        
        let builder = Builder::new(temp.path().to_path_buf(), config);
        
        let source = Path::new("src/main.cay");
        let output = Path::new("target/test.exe");
        
        let args = builder.build_cayc_args(source, output).unwrap();
        
        // 验证参数包含预期内容
        assert!(args.contains(&"-O3".to_string()));
        assert!(args.contains(&"-g".to_string()));
        assert!(args.contains(&"--static".to_string()));
        assert!(args.contains(&"-lm".to_string()));
        assert!(args.contains(&"src/main.cay".to_string()));
        assert!(args.contains(&"target/test.exe".to_string()));
    }
}
