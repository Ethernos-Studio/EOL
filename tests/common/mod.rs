//! Cavvy 语言集成测试公共模块
//!
//! 提供测试辅助函数和工具，被多个测试 crate 共享

use std::process::Command;
use std::fs;
use std::sync::Mutex;
use std::time::Duration;

/// 全局测试锁，确保测试串行执行避免文件冲突
static TEST_LOCK: Mutex<()> = Mutex::new(());

/// 编译并运行单个 EOL 文件，返回输出结果
///
/// 使用 release 版本的 cayc.exe 编译 EOL 源代码为 EXE，
/// 然后执行生成的程序，最后清理生成的临时文件。
///
/// # Arguments
/// * `source_path` - EOL 源代码文件路径（相对于项目根目录）
///
/// # Returns
/// * `Ok(String)` - 成功时返回 stdout 字符串
/// * `Err(String)` - 失败时返回错误信息字符串
///
/// # Example
/// ```rust
/// let output = compile_and_run_eol("examples/hello.cay").expect("编译运行失败");
/// assert!(output.contains("Hello"));
/// ```
///
/// # Notes
/// - 时间复杂度: O(编译时间 + 执行时间)
/// - 会自动清理生成的 .exe 和 .ll 文件
pub fn compile_and_run_eol(source_path: &str) -> Result<String, String> {
    // 使用唯一ID生成输出文件名，避免测试冲突
    let unique_id = format!("{}_{:?}", std::process::id(), std::thread::current().id());
    let exe_path = source_path.replace(".cay", &format!("_{}.exe", unique_id));
    let ir_path = source_path.replace(".cay", &format!("_{}.ll", unique_id));
    
    // 1. 编译 EOL -> EXE (使用 release 版本)
    let output = Command::new("./target/release/cayc.exe")
        .args(&[source_path, &exe_path])
        .output()
        .map_err(|e| format!("Failed to execute cayc: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Compilation failed: {}", stderr));
    }
    
    // 2. 运行生成的 EXE
    let output = Command::new(&exe_path)
        .output()
        .map_err(|e| format!("Failed to execute {}: {}", exe_path, e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Execution failed: {}", stderr));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    
    // 3. 清理生成的文件
    let _ = fs::remove_file(&exe_path);
    let _ = fs::remove_file(&ir_path);
    
    Ok(stdout)
}

/// 编译 EOL 文件，期望编译失败，返回错误信息
///
/// 用于测试应该产生编译错误的代码。
/// 编译失败后返回 stderr 输出，如果编译成功则返回错误。
///
/// # Arguments
/// * `source_path` - EOL 源代码文件路径（相对于项目根目录）
///
/// # Returns
/// * `Ok(String)` - 编译失败时返回 stderr 字符串
/// * `Err(String)` - 编译成功时返回错误
///
/// # Example
/// ```rust
/// let error = compile_eol_expect_error("examples/errors/error_test.cay")
///     .expect("应该编译失败");
/// assert!(error.contains("type mismatch"));
/// ```
pub fn compile_eol_expect_error(source_path: &str) -> Result<String, String> {
    // 使用唯一ID生成输出文件名，避免测试冲突
    let unique_id = format!("{}_{:?}", std::process::id(), std::thread::current().id());
    let exe_path = source_path.replace(".cay", &format!("_{}.exe", unique_id));
    let ir_path = source_path.replace(".cay", &format!("_{}.ll", unique_id));
    
    // 1. 编译 EOL -> EXE (使用 release 版本)
    let output = Command::new("./target/release/cayc.exe")
        .args(&[source_path, &exe_path])
        .output()
        .map_err(|e| format!("Failed to execute cayc: {}", e))?;
    
    // 清理可能生成的文件
    let _ = fs::remove_file(&exe_path);
    let _ = fs::remove_file(&ir_path);
    
    if output.status.success() {
        return Err("Expected compilation to fail, but it succeeded".to_string());
    }
    
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok(stderr)
}

/// 编译并运行 EOL 文件，期望执行失败（用于运行时错误测试），返回错误信息
///
/// 用于测试应该产生运行时错误的代码。
/// 编译成功后执行，如果执行失败返回错误信息，如果执行成功则返回错误。
///
/// # Arguments
/// * `source_path` - EOL 源代码文件路径（相对于项目根目录）
///
/// # Returns
/// * `Ok(String)` - 执行失败时返回错误信息字符串
/// * `Err(String)` - 执行成功时返回错误
///
/// # Example
/// ```rust
/// let error = compile_and_run_expect_error("examples/errors/runtime_error.cay")
///     .expect("应该运行时失败");
/// assert!(error.contains("division by zero"));
/// ```
pub fn compile_and_run_expect_error(source_path: &str) -> Result<String, String> {
    let exe_path = source_path.replace(".cay", ".exe");
    let ir_path = source_path.replace(".cay", ".ll");

    // 1. 编译 EOL -> EXE (使用 release 版本)
    let output = Command::new("./target/release/cayc.exe")
        .args(&[source_path, &exe_path])
        .output()
        .map_err(|e| format!("Failed to execute cayc: {}", e))?;

    if !output.status.success() {
        // 编译失败也返回错误信息
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let _ = fs::remove_file(&exe_path);
        let _ = fs::remove_file(&ir_path);
        return Ok(stderr);
    }

    // 2. 运行生成的 EXE
    let output = Command::new(&exe_path)
        .output()
        .map_err(|e| format!("Failed to execute {}: {}", exe_path, e))?;

    // 3. 清理生成的文件
    let _ = fs::remove_file(&exe_path);
    let _ = fs::remove_file(&ir_path);

    // 如果执行失败（非零退出码），返回错误信息
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        // 合并 stdout 和 stderr，因为错误信息可能输出到 stdout
        let combined = format!("{} {}", stdout, stderr);
        return Ok(format!("runtime error: {}", combined));
    }

    Err("Expected execution to fail, but it succeeded".to_string())
}

/// 断言输出包含所有指定的子字符串
///
/// # Arguments
/// * `output` - 实际的输出字符串
/// * `expected_substrings` - 预期包含的子字符串数组
/// * `test_name` - 测试名称，用于错误信息
pub fn assert_output_contains(output: &str, expected_substrings: &[&str], test_name: &str) {
    for substring in expected_substrings {
        assert!(
            output.contains(substring),
            "{}: Expected output to contain '{}', got: {}",
            test_name,
            substring,
            output
        );
    }
}

/// 断言输出包含任意一个指定的子字符串
///
/// # Arguments
/// * `output` - 实际的输出字符串
/// * `expected_substrings` - 预期包含的子字符串数组（至少包含一个）
/// * `test_name` - 测试名称，用于错误信息
pub fn assert_output_contains_any(output: &str, expected_substrings: &[&str], test_name: &str) {
    let found = expected_substrings.iter().any(|s| output.contains(s));
    assert!(
        found,
        "{}: Expected output to contain at least one of {:?}, got: {}",
        test_name,
        expected_substrings,
        output
    );
}
