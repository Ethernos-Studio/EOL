//! Network 标准库集成测试
//!
//! 测试 Network.cay 库的功能，包括：
//! - Socket 创建和配置
//! - TCP 客户端/服务器
//! - UDP 通信
//! - 网络工具函数

use std::process::Command;
use std::fs;
use std::thread;
use std::time::Duration;

mod common;

/// 测试网络工具函数
#[test]
fn test_network_utils() {
    let output = common::compile_and_run_eol("examples/test_network_utils.cay")
        .expect("编译运行 test_network_utils.cay 失败");
    
    // 验证基本输出
    assert!(output.contains("网络初始化成功"), "应该显示网络初始化成功");
    assert!(output.contains("字节序转换测试通过"), "字节序转换应该通过");
    assert!(output.contains("TCP Socket创建成功"), "TCP Socket应该创建成功");
    assert!(output.contains("UDP Socket创建成功"), "UDP Socket应该创建成功");
    assert!(output.contains("所有测试完成"), "测试应该正常完成");
}

/// 测试 TCP Socket 创建和配置
#[test]
fn test_tcp_socket_creation() {
    // 使用工具测试中的TCP创建测试
    let output = common::compile_and_run_eol("examples/test_network_utils.cay")
        .expect("编译运行失败");
    
    assert!(output.contains("TCP Socket创建成功"), "TCP Socket应该能创建");
    assert!(output.contains("TCP选项设置完成"), "TCP选项应该能设置");
}

/// 测试 UDP Socket 创建和绑定
#[test]
fn test_udp_socket_creation() {
    let output = common::compile_and_run_eol("examples/test_network_utils.cay")
        .expect("编译运行失败");
    
    assert!(output.contains("UDP Socket创建成功"), "UDP Socket应该能创建");
}

/// 测试字节序转换函数
#[test]
fn test_byte_order_conversion() {
    let output = common::compile_and_run_eol("examples/test_network_utils.cay")
        .expect("编译运行失败");
    
    assert!(output.contains("字节序转换测试通过"), "字节序转换应该正确");
}

/// 测试 Socket 地址创建
#[test]
fn test_socket_addr_creation() {
    let output = common::compile_and_run_eol("examples/test_network_utils.cay")
        .expect("编译运行失败");
    
    assert!(output.contains("Socket地址测试"), "应该测试Socket地址");
    assert!(output.contains("创建的地址"), "应该能创建地址");
}

/// 测试 TCP 服务器创建
#[test]
fn test_tcp_server_creation() {
    let output = common::compile_and_run_eol("examples/test_network_utils.cay")
        .expect("编译运行失败");
    
    assert!(output.contains("TCP服务器绑定成功"), "TCP服务器应该能绑定");
    assert!(output.contains("TCP服务器开始监听"), "TCP服务器应该能监听");
}

/// 测试 UDP 通信 (自发自收)
#[test]
fn test_udp_communication() {
    let output = common::compile_and_run_eol("examples/test_network_udp.cay")
        .expect("编译运行 test_network_udp.cay 失败");
    
    assert!(output.contains("网络初始化成功"), "网络应该初始化成功");
    assert!(output.contains("UDP Socket创建成功"), "UDP Socket应该创建成功");
    assert!(output.contains("UDP Socket绑定到端口"), "UDP应该能绑定端口");
    assert!(output.contains("发送成功"), "UDP数据应该能发送");
    assert!(output.contains("收到消息"), "UDP数据应该能接收");
    assert!(output.contains("测试完成"), "测试应该正常完成");
}

/// 测试网络库编译 (不运行，仅检查语法正确性)
#[test]
fn test_network_library_compiles() {
    // 使用 cay-check 检查网络库语法
    let output = Command::new("./target/release/cay-check.exe")
        .args(&["caylibs/Network.cay"])
        .output()
        .expect("执行 cay-check 失败");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // 检查应该通过 (没有错误)
    assert!(
        output.status.success() || (!stderr.contains("error") && !stderr.contains("Error")),
        "Network.cay 应该能通过语法检查\nstderr: {}\nstdout: {}",
        stderr,
        stdout
    );
}

/// 测试 TCP 客户端代码编译
#[test]
fn test_tcp_client_compiles() {
    let output = Command::new("./target/release/cay-check.exe")
        .args(&["examples/test_network_tcp_client.cay"])
        .output()
        .expect("执行 cay-check 失败");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(
        output.status.success() || (!stderr.contains("error") && !stderr.contains("Error")),
        "TCP客户端示例应该能通过语法检查: {}",
        stderr
    );
}

/// 测试 TCP 服务器代码编译
#[test]
fn test_tcp_server_compiles() {
    let output = Command::new("./target/release/cay-check.exe")
        .args(&["examples/test_network_tcp_server.cay"])
        .output()
        .expect("执行 cay-check 失败");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(
        output.status.success() || (!stderr.contains("error") && !stderr.contains("Error")),
        "TCP服务器示例应该能通过语法检查: {}",
        stderr
    );
}

/// 测试 UDP 示例代码编译
#[test]
fn test_udp_example_compiles() {
    let output = Command::new("./target/release/cay-check.exe")
        .args(&["examples/test_network_udp.cay"])
        .output()
        .expect("执行 cay-check 失败");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(
        output.status.success() || (!stderr.contains("error") && !stderr.contains("Error")),
        "UDP示例应该能通过语法检查: {}",
        stderr
    );
}

/// 测试网络工具示例代码编译
#[test]
fn test_network_utils_compiles() {
    let output = Command::new("./target/release/cay-check.exe")
        .args(&["examples/test_network_utils.cay"])
        .output()
        .expect("执行 cay-check 失败");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(
        output.status.success() || (!stderr.contains("error") && !stderr.contains("Error")),
        "网络工具示例应该能通过语法检查: {}",
        stderr
    );
}

/// 测试便捷函数可用性
#[test]
fn test_network_helper_functions() {
    let output = common::compile_and_run_eol("examples/test_network_utils.cay")
        .expect("编译运行失败");
    
    assert!(output.contains("便捷函数 connectTcp 可用"), "connectTcp应该可用");
    assert!(output.contains("便捷函数 listenTcp 可用"), "listenTcp应该可用");
    assert!(output.contains("便捷函数 createUdp 可用"), "createUdp应该可用");
}
